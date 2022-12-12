use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

/// Peer messages type
#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Chocke = 0, // No payload
    Unchoke = 1, // No payload
    Interested = 2, // No payload
    NotInterested = 3, // No payload
    Have = 4,
    Bitfield = 5, // Sent as the first message
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
pub struct Message {
    #[allow(dead_code)]
    pub length: u32,
    pub id: MessageType,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        let length: usize = self.payload.len() + 1;
        let mut buff: Vec<u8> = Vec::with_capacity(length + 4);
        let length: u32 = self.payload.len().try_into().unwrap();
        buff.extend_from_slice(&length.to_be_bytes()[..]);
        buff.push(self.id as u8);
        buff.extend_from_slice(&self.payload);
        buff
    }

    pub fn deserialize(buf_reader: &mut BufReader<&mut TcpStream>) -> Self {
        // Get length of message
        let mut length: [u8; 4] = [0; 4];
        buf_reader.read_exact(&mut length).unwrap();
        let length: u32 = u32::from_be_bytes(length);

        let mut bytes = buf_reader.bytes();

        let id = MessageType::from_u8(bytes.next().unwrap().unwrap());

        let payload: Vec<_> = bytes
            .map(|result| result.unwrap())
            .take(length as usize - 1)
            .collect();

        Message {
            length,
            id,
            payload,
        }
    }
}
