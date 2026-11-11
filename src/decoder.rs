use std::io::{Read, Seek, SeekFrom};

use zerocopy::transmute;

use crate::{ClutType, Error, Picture, PictureInfo, internal::{Header, MipmapHeader, PictureHeader, TIM2_MAGIC}};

pub struct Decoder<R> {
    reader: R,
    header: Header,
    alignment: u8,
    pictures_read: u16,
    next_picture_offset: u64,
}

impl<R: Read + Seek> Decoder<R> {
    pub fn from_reader(mut reader: R) -> Result<Self, Error> {
        let mut header = [0u8; size_of::<Header>()];
        reader.read_exact(&mut header)?;

        let header: Header = transmute!(header);
        if header.magic != TIM2_MAGIC {
            return Err(Error::InvalidSignature);
        }

        let alignment = match header.format {
            0 => 16,
            1 => 128,
            _ => return Err(Error::UnknownFormatId)
        };

        Ok(Self {
            reader,
            header,
            alignment,
            pictures_read: 0,
            next_picture_offset: alignment as _
        })
    }

    pub fn into_reader(self) -> R {
        self.reader
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn picture_count(&self) -> u16 {
        self.header.picture_count.get()
    }

    pub fn next_picture(&mut self) -> Result<Option<PictureDecoder<&mut R>>, Error> {
        if self.pictures_read == self.picture_count() {
            return Ok(None);
        }

        self.reader.seek(SeekFrom::Start(self.next_picture_offset))?;
        let picture = PictureDecoder::from_reader(self.reader.by_ref(), self.alignment as _)?;

        self.next_picture_offset += picture.header.total_size.get() as u64;
        self.pictures_read += 1;

        Ok(Some(picture))
    }
}

pub struct PictureDecoder<R> {
    reader: R,
    header: PictureHeader,
    mipmap_header: Option<MipmapHeader>,
    mipmap_sizes: Vec<u32>,
    user_data: Vec<u8>
}

impl<R: Read + Seek> PictureDecoder<R> {
    pub fn from_reader(mut reader: R, alignment: usize) -> Result<Self, Error> {
        let mut header = [0u8; size_of::<PictureHeader>()];
        reader.read_exact(&mut header)?;

        let header: PictureHeader = transmute!(header);
        let header_size = header.header_size.get();
        let mut mipmap_header = None;
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
                mipmap_header = Some(transmute!(header_buffer));

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
            mipmap_header,
            mipmap_sizes,
            user_data
        })
    }

    pub fn header(&self) -> &PictureHeader {
        &self.header
    }

    pub fn mipmap_header(&self) -> Option<&MipmapHeader> {
        self.mipmap_header.as_ref()
    }

    pub fn mipmap_sizes(&self) -> &[u32] {
        &self.mipmap_sizes
    }
    
    pub fn user_data(&self) -> &[u8] {
        &self.user_data
    }

    /// Returns whether the picture has image data (at least one mipmap level).
    pub fn has_image_data(&self) -> bool {
        self.header.mipmap_count >= 1
    }

    pub fn decode(mut self) -> Result<Picture, Error> {
        if self.header.mipmap_count > 1 {
            return Err(Error::Unimplemented("decoding mipmapped images"));
        }

        let mut image_data = vec![0u8; self.header.image_data_size.get() as _];
        self.reader.read_exact(&mut image_data)?;

        let mut clut_data = vec![0u8; self.header.clut_size.get() as _];
        self.reader.read_exact(&mut clut_data)?;

        let info: PictureInfo = self.header.try_into()?;
        let cluts = if info.clut_type == ClutType::NONE {
            Vec::new()
        }
        else {
            let clut_bytes_per_element = info.clut_type.bytes_per_element()
                .ok_or_else(|| Error::InvalidData("unknown CLUT color type"))?;
            let single_clut_size = self.header.clut_color_count.get() as usize * clut_bytes_per_element;

            clut_data
                .chunks_exact(single_clut_size)
                .map(|s| s.to_vec())
                .collect::<Vec<_>>()
        };

        Picture::from_data(info, image_data, cluts)
    }
}