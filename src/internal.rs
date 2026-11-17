//! TIM2 format details.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, little_endian::*};

use crate::{Error, Info, MipmapInfo, PictureInfo};

pub const TIM2_MAGIC: [u8; 4] = *b"TIM2";
pub const CLUT2_MAGIC: [u8; 4] = *b"CLT2";

pub const MIPMAP_HEADER_SIZES: [usize; 8] = [
    0,  // No image data (CLUT2)
    0,  // Main image only
    32, // LV1
    32, // LV2
    32, // LV3
    48, // LV4
    48, // LV5
    48  // LV6
];

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Header {
    pub magic: [u8; 4],
    pub format_version: u8,
    pub format_id: u8,
    pub picture_count: U16
}

impl Header {
    pub const fn from_info(info: &Info) -> Self {
        Self {
            magic: if info.is_clut2 { CLUT2_MAGIC } else { TIM2_MAGIC },
            format_version: info.format_version,
            format_id: info.format_id as _,
            picture_count: U16::new(info.picture_count)
        }
    }
}

impl TryInto<Info> for Header {
    type Error = Error;

    fn try_into(self) -> Result<Info, Self::Error> {
        Ok(Info {
            is_clut2: match self.magic {
                TIM2_MAGIC => false,
                CLUT2_MAGIC => true,
                _ => return Err(Error::InvalidSignature)
            },
            format_version: self.format_version,
            format_id: self.format_id.try_into()?,
            picture_count: self.picture_count.get()
        })
    }
}

impl From<Info> for Header {
    fn from(info: Info) -> Self {
        Self::from_info(&info)
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

impl TryInto<PictureInfo> for PictureHeader {
    type Error = Error;

    fn try_into(self) -> Result<PictureInfo, Self::Error> {
        Ok(PictureInfo {
            picture_format: self.picture_format,
            mipmap_count: self.mipmap_count,
            clut_type: self.clut_type.try_into()?,
            image_type: self.image_type.try_into()?,
            image_width: self.image_width.get(),
            image_height: self.image_height.get(),
            gs_tex0: self.gs_tex0.get(),
            gs_tex1: self.gs_tex1.get(),
            gs_texafbapabe: self.gs_texafbapabe.get(),
            gs_texclut: self.gs_texclut.get()
        })
    }
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MipmapHeader {
    pub gs_miptbp1: U64,
    pub gs_miptbp2: U64
}

impl MipmapHeader {
    pub const fn to_mipmap_info(&self) -> MipmapInfo {
        MipmapInfo {
            gs_miptbp1: self.gs_miptbp1.get(),
            gs_miptbp2: self.gs_miptbp2.get()
        }
    }

    pub const fn from_mipmap_info(info: &MipmapInfo) -> Self {
        Self {
            gs_miptbp1: U64::new(info.gs_miptbp1),
            gs_miptbp2: U64::new(info.gs_miptbp2)
        }
    }
}

impl Into<MipmapInfo> for MipmapHeader {
    fn into(self) -> MipmapInfo {
        self.to_mipmap_info()
    }
}

impl From<MipmapInfo> for MipmapHeader {
    fn from(info: MipmapInfo) -> Self {
        Self::from_mipmap_info(&info)
    }
}