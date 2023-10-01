use std::{io::SeekFrom, iter::once, ops::SubAssign};

use binrw::{prelude::*, PosValue};

pub mod prelude {
    pub use crate::{
        parse_string, BoundBox, CharInfo, Font, MipTexture, /*MipTextureLevel,*/ Picture, Rgb,
        Vec3,
    };
}

#[binread]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Into<[f32; 3]> for Vec3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[binread]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundBox {
    pub min: Vec3,
    pub max: Vec3,
}

#[binread]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[binread]
#[derive(Debug)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    #[br(count = width * height)]
    pub data: Vec<u8>,
    #[br(temp)]
    colors_used: i16,
    #[br(count = colors_used)]
    pub palette: Vec<Rgb>,
}

#[binread]
#[derive(Debug)]
pub struct MipTexture {
    #[br(temp)]
    begin: PosValue<()>,
    #[br(parse_with = parse_string, args(16))]
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[br(temp)]
    offsets: [u32; 4],

    #[br(if(offsets.iter().all(|&x| x != 0)))]
    #[br(parse_with = indices_parser, args((width * height) as usize, offsets, begin.pos))]
    pub indices: Vec<Vec<u8>>,

    #[br(if(offsets.iter().all(|&x| x != 0)))]
    #[br(seek_before = SeekFrom::Start(begin.pos + 40 + (((width * height) * 85) >> 6) as u64))]
    #[br(temp)]
    colors_used: u16,
    #[br(if(offsets.iter().all(|&x| x != 0)))]
    #[br(count = colors_used)]
    pub palette: Vec<Rgb>,
}

impl MipTexture {
    pub fn pixels(&self, mip_level: usize) -> Option<Vec<u8>> {
        let color_table = &self.palette;
        Some(
            self.indices[mip_level]
                .iter()
                .map(|&i| i as usize)
                .flat_map(|i| {
                    let r = color_table[i].r;
                    let g = color_table[i].g;
                    let b = color_table[i].b;
                    let a = if r < 30 && g < 30 && b > 125 { 0 } else { 255 };
                    once(r).chain(once(g)).chain(once(b)).chain(once(a))
                })
                .collect(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.palette.is_empty()
    }
}

#[binrw::parser(reader, endian)]
fn indices_parser(
    pixels: usize,
    offsets: [u32; 4],
    begin: u64,
) -> BinResult<Vec<Vec<u8>>> {
    let mut indices = Vec::new();
    for i in 0..4 {
        reader.seek(SeekFrom::Start(begin + offsets[i] as u64)).unwrap();
        let mut buf = Vec::new();
        for _ in 0..pixels >> (2 * i) {
            buf.push(<_>::read_options(reader, endian, ()).unwrap());
        }
        indices.push(buf);
    }

    Ok(indices)
}

#[binread]
#[derive(Debug)]
pub struct Font {
    pub width: u32,
    pub height: u32,
    /// Number of fixed-height character rows
    pub row_count: u32,
    /// Height of a row
    pub row_height: u32,
    /// Info about each character
    pub font_info: [CharInfo; 256],
    #[br(count = width * height)]
    pub data: Vec<u8>,
    #[br(temp)]
    colors_used: i16,
    #[br(count = colors_used)]
    pub palette: Vec<Rgb>,
}

#[binread]
#[derive(Debug)]
pub struct CharInfo {
    /// Offset to the character in [Font] data
    pub start_offset: u16,
    pub char_width: u16,
}

#[binrw::parser(reader, endian)]
pub fn parse_string(n: usize) -> BinResult<String> {
    let mut values = Vec::new();

    for _ in 0..n {
        let val = <u8>::read_options(reader, endian, ())?;
        if val != 0 {
            values.push(val);
        }
    }

    Ok(String::from_utf8(values).expect("Couldn't parse string"))
}
