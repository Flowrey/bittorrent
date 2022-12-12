//! MetaInfo module

use serde::{Deserialize, Serialize};

/// Informations about the Torrent
#[derive(Debug, Deserialize, Serialize)]
pub struct Info<'a> {
    /// Suggested name to save the file as.
    pub name: &'a str,

    /// Number of bytes in each piece the file is split into.
    #[serde(rename = "piece length")]
    pub piece_length: u32,

    /// Length of the file in bytes.
    pub length: u32,

    /// String whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20
    /// each of wich is the SHA1 hash of the piece at
    /// the corresponding index.
    #[serde(with = "serde_bytes")]
    pub pieces: &'a [u8],
}

/// Metainfo files (also known as .torrent files)
#[derive(Debug, Deserialize, Serialize)]
pub struct Metainfo<'a> {
    /// The URL of the tracker.
    pub announce: &'a str,

    /// This maps to a Info struct.
    #[serde(borrow)]
    pub info: Info<'a>,
}
