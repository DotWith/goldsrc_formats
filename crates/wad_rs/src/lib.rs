use std::{
    collections::HashMap,
    io::{Cursor, SeekFrom},
};

use binrw::prelude::*;
pub use com_goldsrc_formats::prelude::*;

#[binread]
#[derive(Debug)]
#[br(magic = b"WAD3")]
pub struct Wad {
    #[br(temp)]
    size: u32,
    #[br(temp)]
    offset: u32,
    #[br(seek_before = SeekFrom::Start(offset as u64), parse_with = entries_parser, args(size))]
    pub entiries: HashMap<String, WadEntry>,
}

#[binread]
#[derive(Debug)]
pub struct WadEntry {
    #[br(temp)]
    offset: u32,
    #[br(temp)]
    size: u32,
    #[br(temp, pad_before = 4, pad_after = 3)]
    kind: ContentType,
    #[br(seek_before = SeekFrom::Start(offset as u64), args(kind, size), restore_position)]
    pub content: Content,
    // #[br(seek_before = SeekFrom::Start(offset as u64), restore_position)]
    // #[br(count = size)]
    // pub data: Vec<u8>,
}

#[binread]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[br(repr(u8))]
pub enum ContentType {
    Picture = 0x42,
    MipTexture = 0x43,
    Font = 0x45,
    Other,
}

#[binread]
#[derive(Debug)]
#[br(import(kind: ContentType, size: u32))]
pub enum Content {
    #[br(pre_assert(kind == ContentType::Picture))]
    Picture(Picture),
    #[br(pre_assert(kind == ContentType::MipTexture))]
    MipTexture(MipTexture),
    #[br(pre_assert(kind == ContentType::Font))]
    Font(Font),
    Other {
        #[br(count = size)]
        bytes: Vec<u8>,
    },
}

#[binrw::parser(reader, endian)]
fn entries_parser(n: u32) -> BinResult<HashMap<String, WadEntry>> {
    let mut map = HashMap::new();

    for _ in 0..n {
        let entry = <_>::read_options(reader, endian, ())?;
        let mut name = Vec::new();

        for _ in 0..16 {
            let val = <u8>::read_options(reader, endian, ())?;
            if val != 0 {
                name.push(val);
            }
        }

        map.insert(
            String::from_utf8(name).expect("Couldn't parse string"),
            entry,
        );
    }

    return Ok(map);
}

#[inline]
pub fn read_wad(bytes: &[u8]) -> BinResult<Wad> {
    let mut reader = Cursor::new(bytes);
    let data = reader.read_le()?;
    Ok(data)
}
