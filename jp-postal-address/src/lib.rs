pub mod proto;

pub use proto::jp_postal_code::v1::*;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./_gen/jp_postal_code.file_descriptor.binpb");
