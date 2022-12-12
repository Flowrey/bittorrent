//! BitTorent Protocol Implementation
//!
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

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use url::Url;

pub mod metainfo;
pub mod utils;
mod tracker;

use crate::metainfo::Metainfo;
use crate::utils::urlencode;


// #[derive(Debug, Deserialize, Serialize)]
// struct File<'a> {
//     length: u32,
//     #[serde(borrow)]
//     path: Vec<&'a str>,
// }


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
                println!("Connected to the server");
                let msg = Handshake::new(
                    self.get_info_hash(),
                    "-DE203s-x49Ta1Q*sgGQ".as_bytes().try_into().unwrap(),
                );
                let mut buffer = [0; 512];
                stream.write(&msg.serialize()).unwrap();
                let n = stream.read(&mut buffer).unwrap();
                let received_hanshake = Handshake::deserialize(&buffer[..68]);
                let bitfield_message = Message::deserialize(&buffer[68..n]);

                let offset = bitfield_message.length + 68 + 4;
                let offset: usize = offset.try_into().unwrap();
                let unchoke = Message::deserialize(&buffer[offset..n]);
                println!("{:?}", received_hanshake);
                println!("{:?}", bitfield_message);
                println!("{:?}", unchoke);
            } else {
                println!("Could't connect to server...");
            }
        }
    }
}


/// Peer messages type
#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Chocke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Chocke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have,
            5 => Self::Bitfield,
            6 => Self::Request,
            7 => Self::Piece,
            8 => Self::Cancel,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

/// Peer messages
#[derive(Debug)]
pub struct Message<'a> {
    length: u32,
    id: MessageType,
    payload: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        let length: usize = self.payload.len() + 1;
        let mut buff: Vec<u8> = Vec::with_capacity(length + 4);
        let length: u32 = self.payload.len().try_into().unwrap();
        buff.extend_from_slice(&length.to_be_bytes()[..]);
        buff.push(self.id as u8);
        buff.extend_from_slice(self.payload);
        buff
    }

    pub fn deserialize(bytes: &'a [u8]) -> Self {
        let length: [u8; 4] = bytes[..4].try_into().unwrap();
        let length: u32 = u32::from_be_bytes(length);
        let id = MessageType::from_u8(bytes[4]);
        let payload_length = length as usize - 1;
        let payload = &bytes[5..5 + payload_length];
        Message {
            length,
            id,
            payload,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Handshake {
    length: u8,
    pstr: [u8; 19],
    extensions: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Handshake {
            length: 19,
            pstr: "BitTorrent protocol".as_bytes().try_into().unwrap(),
            extensions: [0u8; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn serialize(&self) -> [u8; 68] {
        let mut buff = [0u8; 68];
        buff[0] = self.length;
        buff[1..20].copy_from_slice(&self.pstr);
        buff[20..28].copy_from_slice(&self.extensions);
        buff[28..48].copy_from_slice(&self.info_hash);
        buff[48..68].copy_from_slice(&self.peer_id);
        buff
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let length = bytes[0];
        let pstr: [u8; 19] = bytes[1..20].try_into().unwrap();
        let extensions: [u8; 8] = bytes[20..28].try_into().unwrap();
        let info_hash: [u8; 20] = bytes[28..48].try_into().unwrap();
        let peer_id: [u8; 20] = bytes[48..68].try_into().unwrap();
        Self {
            length,
            pstr,
            extensions,
            info_hash,
            peer_id,
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
