use std::io::{Read, Seek, SeekFrom};

use zerocopy::transmute;

use crate::{Error, Info, MipmapInfo, Picture, PictureInfo, internal::*};

/// TIM2 decoder.
pub struct Decoder<R> {
    reader: R,
    info: Info,
    pictures_read: u16,
    next_picture_offset: u64,
}

impl<R: Read + Seek> Decoder<R> {
    /// Create a TIM2 decoder from a reader.
    pub fn from_reader(mut reader: R) -> Result<Self, Error> {
        let mut header = [0u8; size_of::<Header>()];
        reader.read_exact(&mut header)?;

        let header: Header = transmute!(header);
        let info: Info = header.try_into()?; // also checks signature

        Ok(Self {
            reader,
            info,
            pictures_read: 0,
            next_picture_offset: info.alignment() as _
        })
    }

    /// Move the reader out of the decoder.
    pub fn into_reader(self) -> R {
        self.reader
    }

    /// Info derived from the header.
    pub const fn info(&self) -> &Info {
        &self.info
    }

    /// Picture count from the header. Determines how many pictures [`next_picture`](Self::next_picture)
    /// will produce.
    pub fn picture_count(&self) -> u16 {
        self.info.picture_count
    }

    /// Read the next picture.
    /// 
    /// Returns `Ok(None)` if there are no more pictures to be read.
    pub fn next_picture(&mut self) -> Result<Option<PictureDecoder<&mut R>>, Error> {
        if self.pictures_read == self.info.picture_count {
            return Ok(None);
        }

        self.reader.seek(SeekFrom::Start(self.next_picture_offset))?;
        let picture = PictureDecoder::from_reader(self.reader.by_ref(), self.info.alignment() as _)?;

        self.next_picture_offset += picture.header.total_size.get() as u64;
        self.pictures_read += 1;

        Ok(Some(picture))
    }
}

/// TIM2 picture decoder.
pub struct PictureDecoder<R> {
    reader: R,
    header: PictureHeader,
    mipmap_info: Option<MipmapInfo>,
    mipmap_sizes: Vec<u32>,
    user_data: Vec<u8>
}

impl<R: Read + Seek> PictureDecoder<R> {
    /// Create a picture decoder from a reader with the specified bytes alignment.
    pub fn from_reader(mut reader: R, alignment: usize) -> Result<Self, Error> {
        let mut header = [0u8; size_of::<PictureHeader>()];
        reader.read_exact(&mut header)?;

        let header: PictureHeader = transmute!(header);
        let header_size = header.header_size.get();
        let mut mipmap_info = None;
        let mut mipmap_sizes = vec![0u32; header.mipmap_count as _];
        let mut user_data = Vec::new();
        if header_size > size_of::<PictureHeader>() as u16 {
            let mut current_offset = size_of::<PictureHeader>();

            if header.mipmap_count > 1 {
                let buffer_len = (size_of::<MipmapHeader>() + mipmap_sizes.len() * size_of::<u32>())
                    .next_multiple_of(alignment);
                let mut buffer = vec![0u8; buffer_len];
                reader.read_exact(&mut buffer)?;

                let (header_buffer, sizes_buffer) = buffer.split_at(size_of::<MipmapHeader>());
                let header_buffer: [u8; size_of::<MipmapHeader>()] = header_buffer.try_into().unwrap();
                let mipmap_header: MipmapHeader = transmute!(header_buffer);
                mipmap_info = Some(mipmap_header.to_mipmap_info());

                for (i, size) in mipmap_sizes.iter_mut().enumerate() {
                    let start = i * size_of::<u32>();
                    let end = start + size_of::<u32>();
                    *size = u32::from_le_bytes(sizes_buffer[start..end].try_into().unwrap());
                }

                current_offset += buffer_len;
            }

            // anything else after this is userdata(?)
            let user_data_size = (header_size as usize).saturating_sub(current_offset);
            if user_data_size > 0 {
                user_data = vec![0u8; user_data_size];
                reader.read_exact(&mut user_data)?;
            }
        }

        Ok(Self {
            reader,
            header,
            mipmap_info,
            mipmap_sizes,
            user_data
        })
    }

    /// Info derived from the mipmap header.
    /// 
    /// Only present when the picture has more than one mipmap.
    pub const fn mipmap_info(&self) -> Option<&MipmapInfo> {
        self.mipmap_info.as_ref()
    }

    pub fn mipmap_sizes(&self) -> &[u32] {
        &self.mipmap_sizes
    }

    pub fn user_data(&self) -> &[u8] {
        &self.user_data
    }

    /// Decode the picture.
    /// 
    /// **Note:** Mipmapped images are not supported yet. [`Error::Unimplemented`] will be returned
    /// when attempting to decode pictures with more than one mipmap.
    pub fn decode(mut self) -> Result<Picture, Error> {
        if self.header.mipmap_count > 1 {
            return Err(Error::Unimplemented("decoding mipmapped images"));
        }

        let mut image_data = vec![0u8; self.header.image_data_size.get() as _];
        self.reader.read_exact(&mut image_data)?;

        let mut clut_data = vec![0u8; self.header.clut_size.get() as _];
        self.reader.read_exact(&mut clut_data)?;

        let clut_color_count = self.header.clut_color_count.get();
        let info: PictureInfo = self.header.try_into()?;

        Picture::new(info, self.user_data, image_data, clut_data, clut_color_count)
    }
}