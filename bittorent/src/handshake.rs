use std::net::TcpStream;
use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, BufReader};
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Handshake {
    length: u8,
    pstr: [u8; 19],
    extensions: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl fmt::Display for Handshake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handshake: {{ pstr:{}, extensions:{}, info_hash:{}, peer_id:{} }}", 
            String::from_utf8_lossy(&self.pstr), 
            String::from_utf8_lossy(&self.extensions), 
            &self.info_hash.encode_hex::<String>(),
            String::from_utf8_lossy(&self.peer_id),
        )
    }
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

    #[allow(dead_code)]
    pub fn deserialize(buf_reader: &mut BufReader<&mut TcpStream>) -> Self {
        let mut bytes = [0; 68];
        buf_reader.read_exact(&mut bytes).unwrap();

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
