//! BitTorent Protocol Implementation
//! BitTorrent is a protocol for distributing files.
//! It identifies content by URL and is designed to integrate
//! seamlessly with the web.
//! Its advantage over plain HTTP is that when multiple downloads
//! of the same file happen concurrently,
//! the downloaders upload to each other,
//! making it possible for the file source to support very
//! large numbers of downloaders with only a modest increase in its load.
//!
//! <https://www.bittorrent.org/beps/bep_0003.html>

use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use url::Url;

pub mod handshake;
pub mod message;
pub mod metainfo;
mod tracker;
pub mod utils;

use crate::handshake::Handshake;
use crate::message::Message;
use crate::metainfo::Metainfo;
use crate::utils::urlencode;

impl<'a> Metainfo<'a> {
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        bendy::serde::from_bytes::<Metainfo>(&bytes).expect("Failed to deserialize torrent file")
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        let bencoded_info = bendy::serde::to_bytes(&self.info).expect("Failed to encode info");
        hasher.update(bencoded_info);
        hasher
            .finalize()
            .try_into()
            .expect("hash must be of size 20")
    }

    pub fn get_peers(&self) -> Vec<SocketAddrV4> {
        let info_hash = urlencode(&self.get_info_hash());

        let mut url = Url::parse(self.announce).expect("Not a valid announce url");
        url.set_query(Some(&format!("info_hash={}", info_hash)));

        let payload = tracker::Request {
            peer_id: "-DE203s-x49Ta1Q*sgGQ",
            port: 58438,
            uploaded: 0,
            downloaded: 0,
            left: self.info.length,
            compact: 1,
        };

        let client = reqwest::blocking::Client::new();
        let res = match client.get(url).query(&payload).send() {
            Ok(r) => r.bytes(),
            Err(e) => panic!("Failed to establish connection to tracker: {}", e),
        };

        let input = res.unwrap();
        let de_res = match bendy::serde::from_bytes::<tracker::Response>(&input) {
            Ok(v) => v,
            Err(_) => panic!("Failed to deserialize tracker response"),
        };

        let chunked_peers = de_res.peers.chunks_exact(6);
        let mut peers: Vec<SocketAddrV4> = Vec::new();
        for peer in chunked_peers {
            let ip: [u8; 4] = peer[..4].try_into().unwrap();
            let ip = Ipv4Addr::from(ip);

            let port: [u8; 2] = peer[4..6].try_into().unwrap();
            let port = u16::from_be_bytes(port);

            let socket = SocketAddrV4::new(ip, port);
            peers.push(socket);
        }
        peers
    }

    pub async fn connect_to_peers(&self, peers: Vec<SocketAddrV4>) {
        for peer in peers {
            self.establish_connection(peer).await;
        }
    }

    async fn establish_connection(&self, peer: SocketAddrV4) {
        if let Ok(mut stream) = TcpStream::connect(peer) {
            println!("Connected to peer: {}", peer);

            // Establish the handshake
            self.establish_handshake(&mut stream);

            // Send intersted & Receive unchocke
            stream.write(&Message::interested().serialize()).unwrap();
            let _unchoke = Message::from_stream(stream.try_clone().unwrap());

            // Create the file
            let mut file = File::create("debian.iso").unwrap();
            // Split the hashes
            let piece_hashes = self.info.pieces.chunks(20);

            // Download eache pieces
            for (index, hash) in piece_hashes.enumerate() {
                self.download_piece(&mut stream, &mut file, index, hash);
            }
        } else {
            println!("Couldn't connect to peer...");
        }
    }

    fn establish_handshake(&self, stream: &mut TcpStream) {
        // Send an handshake
        stream
            .write(&Handshake::new(self.get_info_hash(), "-DE203s-x49Ta1Q*sgGQ").serialize())
            .unwrap();

        // Receive handshake
        let _received_hanshake = Handshake::from_stream(stream.try_clone().unwrap());

        // Receive bitfield
        let _bitfield_message = Message::from_stream(stream.try_clone().unwrap());
        // let bf = bitfield_message.payload;

        // Receive unchocke
        let _unchoke = Message::from_stream(stream.try_clone().unwrap());
    }

    fn download_piece(&self, stream: &mut TcpStream, file: &mut File, index: usize, hash: &[u8]) {
        // Download one piece
        let mut vec_piece: Vec<u8> = Vec::new();
        let mut offset = 0;

        while offset < self.info.piece_length {
            let length: u32 = 2_u32.pow(14);
            // Send request
            stream
                .write(&Message::request(index as u32, offset, length).serialize())
                .unwrap();

            // Receive piece
            let piece = Message::from_stream(stream.try_clone().unwrap());
            let p_index = u32::from_be_bytes(piece.payload.get(0..4).unwrap().try_into().unwrap());
            let p_offset = u32::from_be_bytes(piece.payload.get(4..8).unwrap().try_into().unwrap());
            let p_data = piece.payload.get(8..).unwrap();
            vec_piece.append(&mut p_data.to_vec());
            offset = offset + length;
            println!("Received piece {} at offset {}", p_index, p_offset);
        }

        // Verify the hashs of the downloaded piece
        let mut hasher = Sha1::new();
        hasher.update(&vec_piece);
        let result = hasher.finalize();
        if result[..] == hash[..] {
            println!(
                "Cheksum {} valid for piece {}",
                hash.into_iter().map(|v| format!("{:x}",v)).collect::<String>(),
                index,
            );
        } else {
            panic!("Invalid checksum");
        }

        // Write the piece to the file
        file.write_all(&vec_piece).unwrap();

        // Send we have the piece
        stream
            .write(&Message::have(index as u32).serialize())
            .unwrap();
    }
}

fn _has_piece(bf: &Vec<u8>, index: u32) -> bool {
    let bytes_index = index / 8;
    let offset = index % 8;
    let val = bf[bytes_index as usize] >> (7 - offset) & 1;
    val != 0
}

#[test]
fn test_parsing_metainfo() {
    let data =
        std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let deserialized = Metainfo::from_bytes(&data);
    assert_eq!(
        deserialized.announce,
        "http://bttracker.debian.org:6969/announce"
    );
    assert_eq!(deserialized.info.name, "debian-11.5.0-amd64-netinst.iso");
    assert_eq!(deserialized.info.piece_length, 262144);
    assert_eq!(deserialized.info.length, 400556032);
}

#[test]
fn test_get_peers() {
    let data =
        std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let deserialized = Metainfo::from_bytes(&data);
    let _peers = deserialized.get_peers();
}

#[tokio::test]
async fn test_connecting_to_peers() {
    let data =
        std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let metainfo = Metainfo::from_bytes(&data);
    let peers = [SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 53709)];
    let _connection = metainfo.connect_to_peers(peers.to_vec()).await;
}
