use std::io::{Read, Seek, SeekFrom};

use zerocopy::transmute;

use crate::{Error, Info, MipmapInfo, Picture, PictureMetadata, internal::*};

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
    pub fn next_picture(&mut self) -> Result<Option<Picture>, Error> {
        if self.pictures_read == self.info.picture_count {
            return Ok(None);
        }

        self.reader.seek(SeekFrom::Start(self.next_picture_offset))?;
        let decoder = PictureDecoder::from_reader(self.reader.by_ref())?;
        let total_size = decoder.total_size();
        let picture = decoder.decode()?;

        self.next_picture_offset += total_size as u64;
        self.pictures_read += 1;

        Ok(Some(picture))
    }
}

struct PictureDecoder<R> {
    reader: R,
    header: PictureHeader,
    mipmap_info: Option<MipmapInfo>,
    mipmap_sizes: Vec<u32>,
    user_data: Vec<u8>
}

impl<R: Read + Seek> PictureDecoder<R> {
    pub fn from_reader(mut reader: R) -> Result<Self, Error> {
        let mut header = [0u8; size_of::<PictureHeader>()];
        reader.read_exact(&mut header)?;

        // read header
        let header: PictureHeader = transmute!(header);
        let header_size = header.header_size.get();
        let mipmap_header_size = MIPMAP_HEADER_SIZES.get(header.mipmap_count as usize)
            .copied()
            .ok_or_else(|| Error::InvalidMipmapCount(header.mipmap_count))?;
        let user_data_size = header_size.checked_sub(size_of::<PictureHeader>() as u16 + mipmap_header_size as u16)
            .ok_or_else(|| Error::InvalidData("header size is too small"))?;

        // read mipmap header
        // supports nonstandard format with no mipmaps but non-zero image data size
        let image_data_size = header.image_data_size.get();
        let mut mipmap_sizes = if header.mipmap_count == 1 || (header.mipmap_count == 0 && image_data_size != 0) {
            vec![image_data_size]
        }
        else {
            vec![0u32; header.mipmap_count as _]
        };
        let mipmap_info = if mipmap_header_size != 0 {
            let mut buffer = [0u8; 48]; // mipmap header size is never larger than 48
            reader.read_exact(&mut buffer[..mipmap_header_size])?;

            let (header_buffer, sizes_buffer) = buffer.split_at(size_of::<MipmapHeader>());
            let header_buffer: [u8; size_of::<MipmapHeader>()] = header_buffer.try_into().unwrap();
            let mipmap_header: MipmapHeader = transmute!(header_buffer);

            for (i, size) in mipmap_sizes.iter_mut().enumerate() {
                let offset = i * 4;
                *size = u32::from_le_bytes(sizes_buffer[offset..offset + 4].try_into().unwrap());
            }

            Some(mipmap_header.to_mipmap_info())
        }
        else {
            None
        };

        // read user data
        let user_data = if user_data_size != 0 {
            let mut buffer = vec![0u8; user_data_size as _];
            reader.read_exact(&mut buffer)?;
            buffer
        }
        else {
            Vec::new()
        };

        Ok(Self {
            reader,
            header,
            mipmap_info,
            mipmap_sizes,
            user_data
        })
    }

    pub fn total_size(&self) -> u32 {
        self.header.total_size.get()
    }

    pub fn decode(mut self) -> Result<Picture, Error> {
        let header = self.header;
        if self.mipmap_sizes.iter().sum::<u32>() > header.image_data_size.get() {
            return Err(Error::InvalidData("sum of mipmap sizes larger than image data size"));
        }

        let mut mipmaps = self.mipmap_sizes.into_iter()
            .map(|size| vec![0u8; size as _])
            .collect::<Vec<_>>();
        for image_data in mipmaps.iter_mut() {
            self.reader.read_exact(image_data)?;
        }

        let mut clut_data = vec![0u8; header.clut_size.get() as _];
        self.reader.read_exact(&mut clut_data)?;

        let clut_color_count = header.clut_color_count.get();
        let metadata = PictureMetadata {
            picture_format: header.picture_format,
            clut_type: header.clut_type.try_into()?,
            image_type: header.image_type.try_into()?,
            image_width: header.image_width.get(),
            image_height: header.image_height.get(),
            gs_tex0: header.gs_tex0.get(),
            gs_tex1: header.gs_tex1.get(),
            gs_texafbapabe: header.gs_texafbapabe.get(),
            gs_texclut: header.gs_texclut.get(),
            mipmap_info: self.mipmap_info.into(),
            user_data: self.user_data,
        };

        Picture::new(metadata, mipmaps, clut_data, clut_color_count)
    }
}