use std::io::{Cursor, SeekFrom};

use binrw::prelude::*;
pub use com_goldsrc_formats::prelude::*;

pub const MAX_STUDIO_NAME: usize = 32;

#[binread]
#[derive(Debug)]
#[br(magic = b"IDST", assert(version == 10))]
pub struct Mdl {
    /// Format version
    pub version: u32,
    /// Model name
    #[br(parse_with = parse_string, args(64))]
    pub name: String,
    /// The file size in bytes
    _length: u32,
    /// Ideal eye position
    pub eye_position: Vec3,
    /// Ideal movement hull size
    pub hull: BoundBox,
    /// Clipping bounding box
    pub bounding_box: BoundBox,
    /// Undocumented quake feature flags
    pub flags: u32,

    #[br(parse_with = entry_parser_vec)]
    pub bones: Vec<Bone>,

    #[br(parse_with = entry_parser_vec)]
    pub bone_controller: Vec<BoneController>,

    #[br(parse_with = entry_parser_vec)]
    pub hit_boxes: Vec<IntersectionBox>,

    #[br(parse_with = entry_parser_vec)]
    pub sequences: Vec<Sequence>,

    #[br(parse_with = entry_parser_vec)]
    pub sequence_groups: Vec<SequenceGroup>,

    #[br(parse_with = entry_parser_vec)]
    pub textures: Vec<Texture>,

    #[br(temp)]
    first_texture_data_offset: u32,
    #[br(temp)]
    ref_skins_size: u32,
    #[br(temp)]
    skin_families: u32,
    #[br(temp)]
    ref_skin_offset: u32,

    #[br(parse_with = entry_parser_vec)]
    pub body_parts: Vec<BodyPart>,

    #[br(parse_with = entry_parser_vec)]
    pub attachments: Vec<Attachment>,

    #[br(temp, pad_after = 12)]
    mdl2_offset: u32,

    #[br(seek_before = SeekFrom::Start(mdl2_offset as u64), restore_position)]
    pub mdl2: Mdl2,
    // #[br(parse_with = entry_parser_vec)]
    // pub transitions: Vec<Transition>,
}

#[binread]
#[derive(Debug)]
pub struct Mdl2 {
    /// Number of pose parameters
    pub num_pose_parameters: u32,
    /// Offset to the first pose parameter
    pub pose_param_offset: u32,
    /// Number of IK-autoplaying locks
    pub num_ik_auto_play_locks: u32,
    /// Offset to the first IK-autoplaying lock
    pub ik_auto_play_lock_offset: u32,
    /// Number of IK-chains
    pub num_ik_chains: u32,
    /// Offset to the first IK-chain
    pub ik_chain_offset: u32,
    /// Offset to the first key-value
    pub key_value_offset: u32,
    /// Size of key-values
    pub key_value_size: u32,
    /// Number of hitbox sets
    pub num_hit_box_sets: u32,
    /// Offset to the first hitbox set
    #[br(pad_after = 24)]
    pub hit_box_set_offset: u32,
}

#[binread]
#[derive(Debug)]
pub struct Bone {
    /// The bone name
    #[br(parse_with = parse_string, args(MAX_STUDIO_NAME))]
    pub name: String,
    /// The parent bone offset
    ///
    /// -1 means no parent
    pub parent: u32,
    pub flags: u32,
    /// Avaiable bone controller per motion type
    ///
    /// -1 no controller is avaiable
    pub bone_controller: [u32; 6],
    /// Default position
    pub position: Vec3,
    /// Default rotation
    pub rotation: Vec3,
    /// Compressed position scale
    pub compressed_position_scale: Vec3,
    /// Compressed rotation scale
    pub compressed_rotation_scale: Vec3,
}

#[binread]
#[derive(Debug)]
pub struct BoneController {
    /// Bone affected by this controller
    pub bone: u32,
    /// The motion type
    pub kind: u32,
    /// The minium value
    pub start: f32,
    /// The maxium value
    pub end: f32,
    pub rest: u32,
    /// The bone controller channel
    pub index: u32,
}

#[binread]
#[derive(Debug)]
pub struct IntersectionBox {
    /// The bone this hitbox follows
    pub bone: u32,
    /// The hit group
    pub group: u32,
    /// Extents
    pub extents: BoundBox,
}

#[binread]
#[derive(Debug)]
pub struct Sequence {
    /// The sequence name
    #[br(parse_with = parse_string, args(MAX_STUDIO_NAME))]
    pub label: String,
    /// Frames per second
    pub fps: f32,
    /// Looping/non-looping flags
    pub flags: u32,
    /// The sequence activity
    pub activity: u32,
    /// The sequence activity weight
    pub act_weight: u32,

    #[br(parse_with = entry_parser_vec)]
    pub events: Vec<Event>,

    pub frames: u32,
    #[br(temp)]
    weight_list_offset: u32,
    pub ik_lock_offset: u32,
    pub motion_type: u32,
    pub motion_bone: u32,
    pub linear_movement: f32,
    #[br(temp)]
    auto_layer_offset: u32,
    #[br(temp)]
    key_value_offset: u32,
    /// Extents
    pub extents: BoundBox,
    pub num_blends: u32,
    pub anim_offset: u32,
    pub blend_type: [u32; 2],
    pub blend_start: [f32; 2],
    pub blend_end: [f32; 2],
    pub group_size: [u8; 2],
    pub num_auto_layers: u8,
    pub num_ik_locks: u8,
    pub seq_groups: u8,
    pub entry_node: u32,
    pub exit_node: u32,
    pub node_flags: u8,
    pub cycle_pose_index: u8,
    /// Ideal cross fade in time (0.2 secs default) time = (fadeintime / 100)
    pub fade_in_time: u8,
    /// Ideal cross fade out time (0.2 msecs default)  time = (fadeouttime / 100)
    pub fade_out_time: u8,

    #[br(temp)]
    anim_desc_offset: u32,

    #[br(seek_before = SeekFrom::Start(anim_desc_offset as u64), restore_position)]
    pub anim_desc: AnimDesc,
}

#[binread]
#[derive(Debug)]
#[br(magic = b"IDST")]
pub struct AnimDesc {
    /// Animation label
    #[br(parse_with = parse_string, args(MAX_STUDIO_NAME))]
    pub label: String,
    /// Frames per second
    pub fps: f32,
    pub flags: u32,
    pub frames: u32,
    pub movements: u32,
    pub movement_offset: u32,
    pub ik_rules: u32,
    pub ik_rules_offset: u32,
    pub unused: [u32; 8],
}

#[binread]
#[derive(Debug)]
pub struct Event {
    /// The frame at which this animation event occurs
    pub frame: u32,
    /// The script event type
    pub event: u32,
    pub kind: u32,
    /// Could be path to sound WAVE files
    #[br(parse_with = parse_string, args(64))]
    pub options: String,
}

#[binread]
#[derive(Debug)]
pub struct SequenceGroup {
    /// The textual name
    #[br(parse_with = parse_string, args(MAX_STUDIO_NAME))]
    pub label: String,
    /// The file name
    #[br(parse_with = parse_string, args(64))]
    pub name: String,
    pub cache: u32,
    pub data: u32,
}

#[binread]
#[derive(Debug)]
pub struct Texture {
    /// The file name
    #[br(parse_with = parse_string, args(64))]
    pub name: String,
    /// The texture flags
    pub flags: u32,
    /// Texture width in pixels
    pub width: u32,
    /// Texture height in pixels
    pub height: u32,
    #[br(temp)]
    offset: u32,
}

#[binread]
#[derive(Debug)]
pub struct BodyPart {
    /// The body part name
    #[br(parse_with = parse_string, args(64))]
    pub name: String,
    #[br(temp)]
    models_size: u32,
    pub base: u32,
    #[br(temp)]
    model_offset: u32,

    #[br(seek_before = SeekFrom::Start(model_offset as u64), restore_position, count = models_size)]
    pub models: Vec<Model>,
}

#[binread]
#[derive(Debug)]
pub struct Model {
    /// The model name
    #[br(parse_with = parse_string, args(64))]
    pub name: String,

    pub kind: u32,
    pub bounding_radius: f32,

    #[br(parse_with = entry_parser_vec)]
    pub meshes: Vec<Mesh>,

    #[br(temp)]
    verts_size: u32,

    pub verts_info_offset: u32,

    #[br(temp)]
    verts_offset: u32,

    #[br(seek_before = SeekFrom::Start(verts_offset as u64), restore_position, count = verts_size)]
    pub verts: Vec<Vec3>,

    #[br(temp)]
    norms_size: u32,

    pub norms_info_offset: u32,

    #[br(temp)]
    norms_offset: u32,

    #[br(seek_before = SeekFrom::Start(norms_offset as u64), restore_position, count = norms_size)]
    pub normals: Vec<Vec3>,

    pub blend_vert_info_offset: u32,
    pub blend_norm_info_offset: u32,
}

#[binread]
#[derive(Debug)]
pub struct Mesh {
    #[br(parse_with = entry_parser_vec)]
    pub indices: Vec<Triangle>,
    /// The skin index
    pub skin_ref_index: u32,
    #[br(pad_after = 4)]
    pub norms: u32,
}

#[binread]
#[derive(Debug)]
pub struct Triangle {
    /// Index into vertex array
    pub vert_index: u16,
    /// Index into normal array
    pub norm_index: u16,
    /// Texture coordinates in absoulte space (unnormalized)
    pub s: u16,
    /// Texture coordinates in absoulte space (unnormalized)
    pub t: u16,
}

#[binread]
#[derive(Debug)]
pub struct Attachment {
    #[br(parse_with = parse_string, args(MAX_STUDIO_NAME))]
    pub name: String,
    pub flags: u32,
    /// The bone this attachment follows
    pub bone: u32,
    /// The attachment Origin
    pub org: Vec3,
    /// The attachment vectors
    pub vectors: [Vec3; 3],
}

#[binrw::parser(reader, endian)]
fn entry_parser_vec<T: for<'a> BinRead<Args<'a> = ()> + 'static>() -> BinResult<Vec<T>> {
    let size = u32::read_options(reader, endian, ())?;
    let offset = u32::read_options(reader, endian, ())?;

    let mut map = Vec::new();
    let start_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    for _ in 0..size {
        map.push(<_>::read_options(reader, endian, ())?);
    }

    reader.seek(SeekFrom::Start(start_pos))?;

    Ok(map)
}

#[inline]
pub fn read_mdl(bytes: &[u8]) -> BinResult<Mdl> {
    let mut reader = Cursor::new(bytes);
    let data = reader.read_le()?;
    Ok(data)
}
