use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};
use url::Url;

pub fn urlencode(in_str: &[u8]) -> String {
    let mut escaped_info_hash = String::new();
    for byte in in_str {
        if byte.is_ascii_alphanumeric() || [b'.', b'-', b'_', b'~'].contains(&byte) {
            escaped_info_hash.push(*byte as char);
        } else {
            let str = format!("%{:x}", byte);
            escaped_info_hash.push_str(&str);
        };
    };
    escaped_info_hash
} 

#[derive(Debug, Deserialize, Serialize)]
struct File<'a> {
    length: u32,
    #[serde(borrow)]
    path: Vec<&'a str>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Info<'a> {
    // Suggested name to save the file as
    name: &'a str,

    // Number of bytes in each piece the file is split into
    #[serde(rename = "piece length")]
    piece_length: u32,
    
    // Singe file case
    length: u32,

    // Multiple files case
    // files: Option<Vec<File>>,

    // string whose length is a multiple of 20.
    // It is to be subdivided into strings of length 20
    // each of wich is the SHA1 hash of the piece at
    // the corresponding index
    #[serde(with = "serde_bytes")]
    pieces: &'a [u8],
}

#[derive(Debug, Deserialize, Serialize)]
struct Metainfo<'a> {
    // The URL of the tracker.
    announce: &'a str, 

    #[serde(borrow)]
    info: Info<'a>,
}


impl<'a> Metainfo<'a> {
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        bendy::serde::from_bytes::<Metainfo>(&bytes).expect("Failed to deserialize torrent file")
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        let bencoded_info = bendy::serde::to_bytes(&self.info).expect("Failed to encode info");
        hasher.update(bencoded_info);
        hasher.finalize().try_into().expect("hash must be of size 20")
    }

    pub fn get_escaped_info_hash(&self) -> String {
        let info_hash = self.get_info_hash();
        urlencode(&info_hash)
    }

    pub fn get_peers(&self) -> Vec<SocketAddrV4>  {
        let info_hash = self.get_escaped_info_hash();

        let mut url = Url::parse(self.announce).expect("Not a valid announce url");
        url.set_query(Some(&format!("info_hash={}", info_hash)));

        let payload = TrackerRequest { 
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
        let de_res = match bendy::serde::from_bytes::<TrackerResponse>(&input) {
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
                let msg = Message::new(self.get_info_hash(), "-DE203s-x49Ta1Q*sgGQ".as_bytes().try_into().unwrap());
                let mut buffer = [0; 512];
                stream.write(&msg.serialize()).unwrap();
                let n = stream.read(&mut buffer).unwrap();
                println!("The bytes: {:?}", &buffer[..n]);
            } else {
                println!("Could't connect to server...");
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct TrackerRequest<'a> {
    // 20 bytes sha1 hash of the bencoded from of the
    // info value from the metainfo file.
    // info_hash: &'a str,

    // A string of length 20 wich this downloader used as its id.
    peer_id: &'a str,

    // An optional parameter giving the IP which this peer is at.
    // ip: &'a str,

    // The port number this peer is listening on.
    port: u16,

    // The total amount uploaded so far, encoded in base ten ascii
    uploaded: u32,

    // The total amount downloaded so far
    downloaded: u32,

    // The number of bytes this peer still has to download,
    // encoded in base ten ascii.
    left: u32,

    // This is an optional key which maps to 
    // started, completed, or stopped
    // event: &'a str,

    compact: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TrackerResponse<'a> {
    interval: u32,

    // Compact = 0
    // peers: Vec<Peer>,

    // Compact = 1
    #[serde(with = "serde_bytes")]
    peers: &'a [u8],
}


enum MessageType {
    Chocke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Message {
    length: u8,
    pstr: [u8; 19],
    extensions: [u8; 8],
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}

impl Message {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Message {
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
}

#[test]
fn test_parsing_metainfo() {
    let data = std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let deserialized = Metainfo::from_bytes(&data);
    assert_eq!(deserialized.announce, "http://bttracker.debian.org:6969/announce");
    assert_eq!(deserialized.info.name, "debian-11.5.0-amd64-netinst.iso");
    assert_eq!(deserialized.info.piece_length, 262144);
    assert_eq!(deserialized.info.length, 400556032);
}

#[test]
fn test_get_peers() {
    let data = std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let deserialized = Metainfo::from_bytes(&data);
    let peers = deserialized.get_peers();
}

#[test]
fn test_connecting_to_peers() {
    let data = std::fs::read("debian-11.5.0-amd64-netinst.iso.torrent").expect("Unable to read file");
    let metainfo = Metainfo::from_bytes(&data);
    let peers = [SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6881)];
    let connection = metainfo.connect_to_peers(peers.to_vec());
}
