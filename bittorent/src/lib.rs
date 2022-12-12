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
use std::io::{prelude::*, BufReader};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use url::Url;

pub mod metainfo;
pub mod utils;
pub mod message;
pub mod handshake;
mod tracker;

use crate::metainfo::Metainfo;
use crate::utils::urlencode;
use crate::handshake::Handshake;
use crate::message::{Message, MessageType};


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

    pub fn connect_to_peers(&self, peers: Vec<SocketAddrV4>) {
        for peer in peers {
            if let Ok(mut stream) = TcpStream::connect(peer) {
                println!("Connected to peer: {}", peer);

                // Send an handshake
                let msg = Handshake::new(
                    self.get_info_hash(),
                    "-DE203s-x49Ta1Q*sgGQ".as_bytes().try_into().unwrap(),
                ).serialize();
                stream.write(&msg).unwrap();

                let mut buf = BufReader::new(&mut stream);
                // Receive handshake
                let received_hanshake = Handshake::deserialize(&mut buf);

                // Receive bitfield
                let bitfield_message = Message::deserialize(&mut buf);

                // Receive unchocke
                let unchoke = Message::deserialize(&mut buf);

                // Send interested
                let intersted = Message {
                    length: 1,
                    id: MessageType::Interested,
                    payload: vec![],
                };
                let intersted = intersted.serialize();
                stream.write(&intersted).unwrap();

                println!("{}", received_hanshake);
                println!("{:?}", bitfield_message);
                println!("{:?}", unchoke);
            } else {
                println!("Couldn't connect to peer...");
            }
        }
    }
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

#[test]
fn test_connecting_to_peers() {
    let data =
        std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let metainfo = Metainfo::from_bytes(&data);
    let peers = [SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6881)];
    let _connection = metainfo.connect_to_peers(peers.to_vec());
}
