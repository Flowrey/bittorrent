use serde::{Deserialize, Serialize};

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
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        bendy::serde::from_bytes::<Metainfo>(&bytes).expect("Failed to deserialize torrent file")
    }
}

struct Tracker {
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
