use std::io::Write;

use zerocopy::IntoBytes;

use crate::{Error, FormatId, Info, Picture, internal::{Header, PictureHeader}};

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

        // write aligned header (avoiding an allocation)
        let alignment = match info.format_id {
            FormatId::Align16 => {
                const ALIGNMENT: u8 = FormatId::Align16.alignment();
                let mut buffer = [0u8; ALIGNMENT as _];
                buffer[..size_of::<Header>()].copy_from_slice(header.as_bytes());
                writer.write_all(&buffer)?;

                ALIGNMENT
            },
            FormatId::Align128 => {
                const ALIGNMENT: u8 = FormatId::Align128.alignment();
                let mut buffer = [0u8; ALIGNMENT as _];
                buffer[..size_of::<Header>()].copy_from_slice(header.as_bytes());
                writer.write_all(&buffer)?;

                ALIGNMENT
            }
        };

        Ok(Self {
            writer,
            alignment,
            picture_count: info.picture_count,
            pictures_written: 0
        })
    }

    /// Write the next picture.
    /// 
    /// Returns [`Error::EofExpected`] if the number of pictures written is equals to the
    /// picture count specified in [`Info`].
    pub fn write_picture(&mut self, picture: &Picture) -> Result<(), Error> {
        const PICTURE_HEADER_SIZE: usize = size_of::<PictureHeader>();

        if self.pictures_written == self.picture_count {
            return Err(Error::EofExpected);
        }

        let info = picture.info();
        let user_data = picture.user_data();
        let header_size: u16 = (size_of::<PictureHeader>() + user_data.len())
            .next_multiple_of(self.alignment as _)
            .try_into()?;
        let image_data_size: u32 = picture.image_data().len().try_into()?;
        let clut_size: u32 = picture.clut_data().len().try_into()?;
        let total_size = (header_size as u32)
            .checked_add(image_data_size)
            .and_then(|size| size.checked_add(clut_size))
            .ok_or_else(|| Error::InvalidData("total picture size exceeds u32 limit"))?;
        let clut_color_count: u16 = picture.clut_color_count().try_into()?;

        let header = PictureHeader {
            total_size: total_size.into(),
            clut_size: clut_size.into(),
            image_data_size: image_data_size.into(),
            header_size: header_size.into(),
            clut_color_count: clut_color_count.into(),
            picture_format: info.picture_format,
            mipmap_count: info.mipmap_count,
            clut_type: info.clut_type.bits(),
            image_type: info.image_type as _,
            image_width: info.image_width.into(),
            image_height: info.image_height.into(),
            gs_tex0: info.gs_tex0.into(),
            gs_tex1: info.gs_tex1.into(),
            gs_texafbapabe: info.gs_texafbapabe.into(),
            gs_texclut: info.gs_texclut.into(),
        };

        let mut header_buffer = vec![0u8; header_size as _];
        header_buffer[..PICTURE_HEADER_SIZE].copy_from_slice(header.as_bytes());
        header_buffer[PICTURE_HEADER_SIZE..PICTURE_HEADER_SIZE + user_data.len()].copy_from_slice(user_data);

        self.writer.write_all(&header_buffer)?;
        self.writer.write_all(picture.image_data())?;
        self.writer.write_all(picture.clut_data())?;

        self.pictures_written += 1;
        Ok(())
    }
}