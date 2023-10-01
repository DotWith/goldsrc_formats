use std::{
    io::{Cursor, SeekFrom},
    mem::size_of,
    ops::Range,
};

use binrw::prelude::*;
pub use valve_fmt_shared::prelude::*;

pub use cubemap::*;
pub use entities::*;

mod cubemap;
mod entities;

#[binread]
#[derive(Debug)]
#[br(assert(version == 30))]
pub struct Bsp {
    pub version: u32,

    #[br(parse_with = entry_parser_entities)]
    pub entities: Vec<Entity>,

    #[br(parse_with = entry_parser_vec)]
    pub planes: Vec<Plane>,

    #[br(parse_with = entry_parser_textures)]
    pub textures: Vec<MipTexture>,
    // pub tex_offset: u32,
    // pub tex_size: u32,
    #[br(parse_with = entry_parser_vec)]
    pub vertices: Vec<Vec3>,

    pub vis_offset: u32,
    pub vis_size: u32,

    #[br(parse_with = entry_parser_vec)]
    pub nodes: Vec<Node>,

    #[br(parse_with = entry_parser_vec)]
    pub texture_infos: Vec<TextureInfo>,

    #[br(parse_with = entry_parser_vec)]
    pub faces: Vec<Face>,

    #[br(parse_with = entry_parser_vec)]
    pub lightmap: Vec<u8>,

    #[br(parse_with = entry_parser_vec)]
    pub clip_nodes: Vec<ClipNode>,

    #[br(parse_with = entry_parser_vec)]
    pub leaves: Vec<Leaf>,

    #[br(parse_with = entry_parser_vec)]
    pub mark_surfaces: Vec<u16>,

    #[br(parse_with = entry_parser_vec_range)]
    pub edges: Vec<Range<u16>>,

    #[br(parse_with = entry_parser_vec)]
    pub surf_edges: Vec<i32>,

    #[br(parse_with = entry_parser_vec)]
    pub models: Vec<Model>,
}

impl Bsp {
    pub fn normals(&self) -> Vec<Vec3> {
        self.planes.iter().map(|p| p.normal).collect()
    }
}

#[binread]
#[derive(Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub dist: f32,
    pub kind: PlaneType,
}

#[binread]
#[derive(Debug)]
#[br(repr(u32))]
pub enum PlaneType {
    /// Plane is perpendicular to given axis
    X,
    Y,
    Z,
    /// Non-axial plane is snapped to the nearest
    AnyX,
    AnyY,
    AnyZ,
}

#[binread]
#[derive(Debug)]
pub struct Node {
    /// Index into planes lump
    pub plane_index: u32,
    pub children: [i16; 2],
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    /// Index into faces
    pub first_face: u16,
    /// Count into faces
    pub num_faces: u16,
}

#[binread]
#[derive(Debug)]
pub struct TextureInfo {
    pub s: Vec3,
    pub s_shift: f32,
    pub t: Vec3,
    pub t_shift: f32,
    pub miptex: u32,
    pub flags: u32,
}

#[binread]
#[derive(Debug)]
pub struct Face {
    pub plane_index: u16,
    pub plane_side: u16,
    pub first_edge: u32,
    pub edges: u16,
    pub texture_info: u16,
    pub styles: [u8; 4],
    pub lightmap_offset: u32,
}

#[binread]
#[derive(Debug)]
pub struct ClipNode {
    pub plane_index: i32,
    pub children: [i16; 2],
}

#[binread]
#[derive(Debug)]
pub struct Leaf {
    pub contents: LeafContent,
    pub vis_offset: i32,
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub first_mark_surface: u16,
    pub mark_surface: u16,
    pub ambient_levels: [u8; 4],
}

#[binread]
#[derive(Debug)]
#[br(repr(i32))]
pub enum LeafContent {
    Empty = -1,
    Solid = -2,
    Water = -3,
    Slime = -4,
    Lava = -5,
    Sky = -6,
    Origin = -7,
    Clip = -8,
    Current0 = -9,
    Current90 = -10,
    Current180 = -11,
    Current270 = -12,
    CurrentUp = -13,
    CurrentDown = -14,
    Translucent = -15,
}

#[binread]
#[derive(Debug)]
pub struct Model {
    pub bounding_box: BoundBox,
    pub origin: Vec3,
    pub head_nodes: [i32; 4],
    pub vis_leafs: i32,
    pub first_face: i32,
    pub faces: i32,
}

#[binrw::parser(reader, endian)]
pub fn entry_parser_entities() -> BinResult<Vec<Entity>> {
    let offset = u32::read_options(reader, endian, ())?;
    let size = u32::read_options(reader, endian, ())?;

    let mut values = Vec::new();
    let start_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..size {
        let val = <u8>::read_options(reader, endian, ())?;
        if val != 0 {
            values.push(val);
        }
    }

    reader.seek(SeekFrom::Start(start_pos))?;

    let string = String::from_utf8(values).expect("Couldn't parse string");

    Ok(parse_entities(&string))
}

#[binrw::parser(reader, endian)]
fn entry_parser_vec_range() -> BinResult<Vec<Range<u16>>> {
    let offset = u32::read_options(reader, endian, ())?;
    let size = u32::read_options(reader, endian, ())?;

    let mut map = Vec::new();
    let start_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..size as usize / size_of::<Range<u16>>() {
        map.push(Range {
            start: <_>::read_options(reader, endian, ())?,
            end: <_>::read_options(reader, endian, ())?,
        });
    }

    reader.seek(SeekFrom::Start(start_pos))?;

    Ok(map)
}

#[binrw::parser(reader, endian)]
fn entry_parser_textures() -> BinResult<Vec<MipTexture>> {
    let start_offset = u32::read_options(reader, endian, ())?;
    let _size = u32::read_options(reader, endian, ())?;

    let start_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(start_offset as u64))?;

    let num = u32::read_options(reader, endian, ())?;
    let mut offsets = Vec::new();

    for _ in 0..num {
        offsets.push(u32::read_options(reader, endian, ())?);
    }

    let mut mip_textures = Vec::new();
    for offset in offsets {
        reader.seek(SeekFrom::Start((start_offset + offset) as u64))?;
        mip_textures.push(<_>::read_options(reader, endian, ())?);
    }

    reader.seek(SeekFrom::Start(start_pos))?;

    Ok(mip_textures)
}

#[binrw::parser(reader, endian)]
fn entry_parser_vec<T: for<'a> BinRead<Args<'a> = ()> + 'static>() -> BinResult<Vec<T>> {
    let offset = u32::read_options(reader, endian, ())?;
    let size = u32::read_options(reader, endian, ())?;

    let mut map = Vec::new();
    let start_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..size as usize / size_of::<T>() {
        map.push(<_>::read_options(reader, endian, ())?);
    }

    reader.seek(SeekFrom::Start(start_pos))?;

    Ok(map)
}

#[inline]
pub fn read_bsp(bytes: &[u8]) -> BinResult<Bsp> {
    let mut reader = Cursor::new(bytes);
    let data = reader.read_le()?;
    Ok(data)
}
