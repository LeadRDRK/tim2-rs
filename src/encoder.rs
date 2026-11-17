use std::io::Write;

use zerocopy::IntoBytes;

use crate::{Error, Info, Picture, internal::{Header, PictureHeader}};

const ZEROS: [u8; 128] = [0u8; 128];
fn padding(size: usize, alignment: u8) -> &'static [u8] {
    let alignment = alignment as usize;
    &ZEROS[..0usize.wrapping_sub(size) & (alignment - 1)]
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

    /// Write the next picture.
    /// 
    /// Returns [`Error::EofExpected`] if the number of pictures written is equals to the
    /// picture count specified in [`Info`].
    pub fn write_picture(&mut self, picture: &Picture) -> Result<(), Error> {
        if self.pictures_written == self.picture_count {
            return Err(Error::EofExpected);
        }

        let info = picture.info();
        let orig_header_size = size_of::<PictureHeader>() + picture.user_data().len();

        let (header_size, header_padding) = self.aligned_size_and_padding::<u16>(orig_header_size)
            .ok_or_else(|| Error::InvalidData("user data too large"))?;

        let (image_data_size, image_data_padding) = self.aligned_size_and_padding(picture.image_data().len())
            .ok_or_else(|| Error::InvalidData("image data too large"))?;

        let (clut_size, clut_padding) = self.aligned_size_and_padding(picture.clut_data().len())
            .ok_or_else(|| Error::InvalidData("CLUT data too large"))?;

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

        self.writer.write_all(header.as_bytes())?;
        self.writer.write_all(picture.user_data())?;
        self.writer.write_all(header_padding)?;

        self.writer.write_all(picture.image_data())?;
        self.writer.write_all(image_data_padding)?;

        self.writer.write_all(picture.clut_data())?;
        self.writer.write_all(clut_padding)?;

        self.pictures_written += 1;
        Ok(())
    }

    fn aligned_size_and_padding<T: TryFrom<usize>>(&self, size: usize) -> Option<(T, &'static [u8])> {
        let alignment = self.alignment as usize;
        let aligned = (size.checked_add(alignment)? - 1) & !(alignment - 1);
        Some((aligned.try_into().ok()?, &ZEROS[..aligned - size]))
    }
}