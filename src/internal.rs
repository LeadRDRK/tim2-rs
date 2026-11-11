use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, little_endian::*};

use crate::Info;

pub const TIM2_MAGIC: [u8; 4] = *b"TIM2";

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u8,
    pub format: u8,
    pub picture_count: U16
}

impl Header {
    pub fn to_info(&self) -> Info {
        Info {
            version: self.version,
            format: self.format,
            picture_count: self.picture_count.get()
        }
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct PictureHeader {
    pub total_size: U32,
    pub clut_size: U32,
    pub image_data_size: U32,
    pub header_size: U16,
    pub clut_color_count: U16,
    pub picture_format: u8,
    pub mipmap_count: u8,
    pub clut_type: u8,
    pub image_type: u8,
    pub image_width: U16,
    pub image_height: U16,
    pub gs_tex0: U64,
    pub gs_tex1: U64,
    pub gs_texafbapabe: U32,
    pub gs_texclut: U32
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MipmapHeader {
    pub gs_miptbp1: U64,
    pub gs_miptbp2: U64
}