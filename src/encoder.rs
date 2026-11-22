use std::io::Write;

use zerocopy::IntoBytes;

use crate::{Error, Info, Picture, internal::*};

const ZEROS: [u8; 128] = [0u8; 128];
fn padding(size: usize, alignment: u8) -> &'static [u8] {
    let alignment = alignment as usize;
    &ZEROS[..0usize.wrapping_sub(size) & (alignment - 1)]
}

fn aligned_size(size: usize, alignment: u8) -> Option<usize> {
    let alignment = alignment as usize;
    Some((size.checked_add(alignment)? - 1) & !(alignment - 1))
}

/// TIM2 encoder.
pub struct Encoder<W> {
    writer: W,
    alignment: u8,
    picture_count: u16,
    pictures_written: u16
}

impl<W: Write> Encoder<W> {
    /// Create a TIM2 encoder from a writer.
    /// 
    /// The header will be written. After this, you should call [`write_picture`](Self::write_picture)
    /// as many times as the picture count.
    pub fn from_writer(mut writer: W, info: &Info) -> Result<Self, Error> {
        let header = Header::from_info(info);

        // write aligned header
        let alignment = info.format_id.alignment();
        writer.write_all(header.as_bytes())?;
        writer.write_all(padding(size_of::<Header>(), alignment))?;

        Ok(Self {
            writer,
            alignment,
            picture_count: info.picture_count,
            pictures_written: 0
        })
    }

    /// Move the writer out of the decoder.
    pub fn into_writer(self) -> W {
        self.writer
    }

    /// Picture count from [`Info`].
    pub fn picture_count(&self) -> u16 {
        self.picture_count
    }

    /// Number of pictures written so far.
    pub fn pictures_written(&self) -> u16 {
        self.pictures_written
    }

    /// Write the next picture.
    /// 
    /// Returns [`Error::EofExpected`] if the number of pictures written is equals to the
    /// picture count specified in [`Info`].
    pub fn write_picture(&mut self, picture: &Picture) -> Result<(), Error> {
        if self.pictures_written == self.picture_count {
            return Err(Error::EofExpected);
        }

        let metadata = picture.metadata();

        let mipmap_count = picture.mipmap_count();
        let mipmap_header_size = MIPMAP_HEADER_SIZES[mipmap_count as usize];
        let header_size = size_of::<PictureHeader>() + mipmap_header_size + metadata.user_data.len();
        let (header_size, header_padding) = self.aligned_size_and_padding::<u16>(header_size)
            .ok_or_else(|| Error::InvalidData("user data too large"))?;

        let mut mipmap_sizes = [0usize; 7];
        let mut image_data_size = 0usize;
        for (mipmap, size) in picture.mipmaps().iter().zip(mipmap_sizes.iter_mut()) {
            *size = aligned_size(mipmap.len(), 16) // always aligned to 16 bytes
                .ok_or_else(|| Error::InvalidData("image data too large"))?;
            image_data_size += *size;
        }
        let (image_data_size, image_data_padding) = self.aligned_size_and_padding(image_data_size)
            .ok_or_else(|| Error::InvalidData("image data too large"))?;

        let (clut_size, clut_padding) = self.aligned_size_and_padding(picture.clut_data().len())
            .ok_or_else(|| Error::InvalidData("CLUT data too large"))?;

        let total_size = (header_size as u32)
            .checked_add(image_data_size)
            .and_then(|size| size.checked_add(clut_size))
            .ok_or_else(|| Error::InvalidData("total picture size exceeds u32 limit"))?;

        let clut_color_count: u16 = picture.clut_color_count().try_into()
            .map_err(|_| Error::InvalidData("number of CLUT elements exceeds u16 limit"))?;

        let header = PictureHeader {
            total_size: total_size.into(),
            clut_size: clut_size.into(),
            image_data_size: image_data_size.into(),
            header_size: header_size.into(),
            clut_color_count: clut_color_count.into(),
            picture_format: metadata.picture_format,
            mipmap_count,
            clut_type: metadata.clut_type.bits(),
            image_type: metadata.image_type as _,
            image_width: metadata.image_width.into(),
            image_height: metadata.image_height.into(),
            gs_tex0: metadata.gs_tex0.into(),
            gs_tex1: metadata.gs_tex1.into(),
            gs_texafbapabe: metadata.gs_texafbapabe.into(),
            gs_texclut: metadata.gs_texclut.into(),
        };

        self.writer.write_all(header.as_bytes())?;
        if mipmap_header_size != 0 {
            let mut buffer = [0u8; 48]; // mipmap header size is never larger than 48
            {
                let (header_buffer, sizes_buffer) = buffer.split_at_mut(size_of::<MipmapHeader>());
                let header = metadata.mipmap_info.as_ref()
                    .map(MipmapHeader::from_mipmap_info)
                    .ok_or_else(|| Error::NoMipmapInfo)?;
                header_buffer.copy_from_slice(header.as_bytes());

                for (i, chunk) in picture.mipmaps().iter().enumerate() {
                    let offset = i * 4;
                    sizes_buffer[offset..offset + 4].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
                }
            }
            self.writer.write_all(&buffer[..mipmap_header_size])?;
        }
        self.writer.write_all(&metadata.user_data)?;
        self.writer.write_all(header_padding)?;

        for (mipmap, size) in picture.mipmaps().iter().zip(mipmap_sizes.into_iter()) {
            self.writer.write_all(&mipmap)?;
            self.writer.write_all(&ZEROS[..size - mipmap.len()])?; // per-mipmap alignment
        }
        self.writer.write_all(image_data_padding)?;

        self.writer.write_all(picture.clut_data())?;
        self.writer.write_all(clut_padding)?;

        self.pictures_written += 1;
        Ok(())
    }

    fn aligned_size<T: TryFrom<usize>>(&self, size: usize) -> Option<T> {
        aligned_size(size, self.alignment)
            .and_then(|aligned| aligned.try_into().ok())
    }

    fn aligned_size_and_padding<T: TryFrom<usize>>(&self, size: usize) -> Option<(T, &'static [u8])> {
        self.aligned_size::<usize>(size)
            .and_then(|aligned| Some(
                (aligned.try_into().ok()?, &ZEROS[..aligned - size])
            ))
    }
}