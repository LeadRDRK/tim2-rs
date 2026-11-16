//! TIM2 format details.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, little_endian::*};

use crate::{Error, Info, MipmapInfo, PictureInfo};

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
    pub const fn to_info(&self) -> Info {
        Info {
            version: self.version,
            format: self.format,
            picture_count: self.picture_count.get()
        }
    }

    pub const fn from_info(info: &Info) -> Self {
        Self {
            magic: TIM2_MAGIC,
            version: info.version,
            format: info.format,
            picture_count: U16::new(info.picture_count)
        }
    }
}

impl Into<Info> for Header {
    fn into(self) -> Info {
        self.to_info()
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
}

impl Into<MipmapInfo> for MipmapHeader {
    fn into(self) -> MipmapInfo {
        self.to_mipmap_info()
    }
}