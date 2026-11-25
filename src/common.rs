use std::{borrow::Cow, ops::{Deref, DerefMut}};

use bitflags::bitflags;

use crate::Error;

/// TIM2 info. Derived from the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    /// Whether the format signature is for the CLUT2 format.
    /// In other words, this will be `false` for TIM2 files.
    pub is_clut2: bool,
    /// Format version (3 or 4).
    pub format_version: u8,
    pub format_id: FormatId,
    pub picture_count: u16
}

impl Info {
    /// Byte alignment of the format.
    pub const fn alignment(&self) -> u8 {
        self.format_id.alignment()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum FormatId {
    Align16,
    Align128
}

impl FormatId {
    pub const fn alignment(&self) -> u8 {
        match self {
            FormatId::Align16 => 16,
            FormatId::Align128 => 128
        }
    }
}

impl TryFrom<u8> for FormatId {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Align16,
            1 => Self::Align128,
            _ => return Err(Error::InvalidFormatId(value))
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ClutType: u8 {
        // HACK: reverse order for the Debug impl to show the correct color type
        const RGB32 = 0b00000011;
        const RGB24 = 0b00000010;
        const RGB16 = 0b00000001;
        const CSM2  = 0b10000000;

        /// PAL_NONE (no palette)
        const NONE = 0;
        /// PAL_RGB16_CSM1 (16-bit palette, swizzled)
        const RGB16_CSM1 = Self::RGB16.bits();
        /// PAL_RGB24_CSM1 (24-bit palette, swizzled)
        const RGB24_CSM1 = Self::RGB24.bits();
        /// PAL_RGB32_CSM1 (32-bit palette, swizzled)
        const RGB32_CSM1 = Self::RGB32.bits();
        /// PAL_RGB16_CSM2 (16-bit palette, linear)
        const RGB16_CSM2 = Self::RGB16.bits() | Self::CSM2.bits();
        /// PAL_RGB24_CSM2 (24-bit palette, linear)
        const RGB24_CSM2 = Self::RGB24.bits() | Self::CSM2.bits();
        /// PAL_RGB32_CSM2 (32-bit palette, linear)
        const RGB32_CSM2 = Self::RGB32.bits() | Self::CSM2.bits();

        // allow any other bit to be set to handle invalid values
        const _ = !0;
    }
}

impl ClutType {
    /// Extract the color type bits from this CLUT type.
    pub const fn color_type(self) -> ClutType {
        self.difference(Self::CSM2)
    }

    /// Check if the CLUT type is linear (CSM2).
    pub const fn is_linear(&self) -> bool {
        self.contains(Self::CSM2)
    }

    /// Bits per element for this CLUT type.
    pub const fn bits_per_element(&self) -> Option<usize> {
        Some(match self.color_type() {
            ClutType::RGB16 => 16,
            ClutType::RGB24 => 24,
            ClutType::RGB32 => 32,
            _ => return None
        })
    }

    /// Bytes per element for this CLUT type.
    pub const fn bytes_per_element(&self) -> Option<usize> {
        Some(match self.color_type() {
            ClutType::RGB16 => 2,
            ClutType::RGB24 => 3,
            ClutType::RGB32 => 4,
            _ => return None
        })
    }
}

impl TryFrom<u8> for ClutType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::NONE,
            1 => Self::RGB16_CSM1,
            2 => Self::RGB24_CSM1,
            3 => Self::RGB32_CSM1,
            129 => Self::RGB16_CSM2,
            130 => Self::RGB24_CSM2,
            131 => Self::RGB32_CSM2,
            _ => return Err(Error::InvalidClutType(value))
        })
    }
}

/// Picture image data type,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ImageType {
    /// TIM2_RGB16 (16bpp image, RGBA5551)
    Rgb16 = 1,
    /// TIM2_RGB24 (24bpp image, RGB888)
    Rgb24 = 2,
    /// TIM2_RGB32 (32bpp image, RGBA8888)
    Rgb32 = 3,
    /// TIM2_IDTEX4 (4-bit indexed image, PAL4)
    IdTex4 = 4,
    /// TIM2_IDTEX8 (8-bit indexed image, PAL8)
    IdTex8 = 5
}

impl ImageType {
    /// Bits per pixel for this image type.
    pub const fn bits_per_pixel(&self) -> usize {
        match self {
            ImageType::Rgb16  => 16,
            ImageType::Rgb24  => 24,
            ImageType::Rgb32  => 32,
            ImageType::IdTex4 => 4,
            ImageType::IdTex8 => 8,
        }
    }

    /// Bytes per `n` pixels for this image type.
    /// 
    /// Since [`ImageType::IdTex4`] uses only 4 bits for a pixel, it is not safe to assume that
    /// `bytes_per_n_pixels(n) == n * bytes_per_n_pixels(1)` for all types.
    pub const fn bytes_per_n_pixels(&self, n: usize) -> usize {
        match self {
            ImageType::Rgb16  => n * 2,
            ImageType::Rgb24  => n * 3,
            ImageType::Rgb32  => n * 4,
            ImageType::IdTex4 => n.div_ceil(2),
            ImageType::IdTex8 => n,
        }
    }

    /// Check if the image type is indexed.
    pub const fn is_indexed(&self) -> bool {
        match self {
            ImageType::IdTex4 => true,
            ImageType::IdTex8 => true,
            _ => false
        }
    }
}

impl TryFrom<u8> for ImageType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Rgb16,
            2 => Self::Rgb24,
            3 => Self::Rgb32,
            4 => Self::IdTex4,
            5 => Self::IdTex8,
            _ => return Err(Error::InvalidImageType(value))
        })
    }
}

/// Picture metadata. Derived from the picture header.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PictureMetadata {
    /// Should always be 0.
    pub picture_format: u8,
    pub clut_type: ClutType,
    pub image_type: ImageType,
    pub image_width: u16,
    pub image_height: u16,
    /// GS TEX0 register data.
    pub gs_tex0: u64,
    /// GS TEX1 register data.
    pub gs_tex1: u64,
    /// GS TEXAFBAPABE registers data.
    pub gs_texafbapabe: u32,
    /// GS TEXCLUT register data.
    pub gs_texclut: u32,
    pub mipmap_info: Option<MipmapInfo>,
    /// Raw trailing data right after the header.
    pub user_data: Vec<u8>
}

/// TIM2 picture.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Picture {
    metadata: PictureMetadata,
    mipmaps: Vec<Vec<u8>>,
    clut_data: Vec<u8>,
    clut_len: usize
}

impl Picture {
    /// Create a new picture from raw data.
    /// 
    /// The CLUT data may contain multiple CLUTs. The length for each CLUT is determined
    /// by the CLUT color count.
    /// 
    /// The image data and CLUT data length will be validated against the image type, image size,
    /// CLUT type and CLUT color count. The data is considered valid if it's large enough.
    /// Trailing data is ignored and will be kept as-is.
    /// Additionally, there must not be more than 7 mipmaps, including the base image.
    pub fn new(
        metadata: PictureMetadata,
        mipmaps: Vec<Vec<u8>>,
        clut_data: impl Into<Vec<u8>>,
        clut_color_count: u16
    ) -> Result<Self, Error> {
        if mipmaps.len() > 7 {
            return Err(Error::InvalidData("more than 7 mipmaps"));
        }

        let base_width = metadata.image_width as usize;
        let base_height = metadata.image_height as usize;
        for (i, data) in mipmaps.iter().enumerate() {
            let image_pixels = (base_width >> i) * (base_height >> i);
            let image_data_len = metadata.image_type.bytes_per_n_pixels(image_pixels);
            if data.len() < image_data_len {
                return Err(Error::InvalidData("insufficient image data length"));
            }
        }

        let clut_data = clut_data.into();
        let clut_len = if metadata.clut_type != ClutType::NONE {
            let clut_bytes_per_element = metadata.clut_type.bytes_per_element()
                .ok_or_else(|| Error::InvalidData("unknown CLUT color type"))?;

            // OK: this will still allow empty clut data when there's a clut type
            let clut_len = clut_color_count as usize * clut_bytes_per_element;
            if clut_data.len() < clut_len {
                return Err(Error::InvalidData("insufficient CLUT data length"));
            }
            clut_len
        }
        else {
            0
        };

        Ok(Self {
            metadata,
            mipmaps,
            clut_data,
            clut_len
        })
    }

    pub const fn metadata(&self) -> &PictureMetadata {
        &self.metadata
    }

    /// Raw mipmaps data.
    pub fn mipmaps(&self) -> &[Vec<u8>] {
        &self.mipmaps
    }

    pub fn mipmap_count(&self) -> u8 {
        self.mipmaps.len() as _
    }

    /// Get the raw image data for the specified mipmap level.
    pub fn image_data(&self, mipmap: u8) -> Option<&[u8]> {
        self.mipmaps.get(mipmap as usize).map(|v| &**v)
    }

    /// Get the CLUT at the specified index.
    pub fn clut(&self, index: usize) -> Option<Clut<'_>> {
        if self.clut_len == 0 { return None; }

        let clut_type = self.metadata.clut_type;
        let color_format: ColorFormat = clut_type.try_into().unwrap(); // should never panic if clut_len != 0
        let offset = index * self.clut_len;
        self.clut_data
            .get(offset..offset + self.clut_len)
            .map(|data| Clut::new_borrowed(data, color_format, clut_type.is_linear()))
    }

    /// Get an iterator of all the CLUTs in this picture.
    pub fn cluts(&self) -> impl Iterator<Item = Clut<'_>> {
        let clut_type = self.metadata.clut_type;
        let color_format: Option<ColorFormat> = clut_type.try_into().ok();
        std::iter::once(&self.clut_data)
            .filter(|_| self.clut_len != 0)
            .flat_map(|data| data.chunks_exact(self.clut_len))
            .map(move |chunk| Clut::new_borrowed(chunk, color_format.unwrap(), clut_type.is_linear()))
    }

    /// Raw CLUT data.
    pub fn clut_data(&self) -> &[u8] {
        &self.clut_data
    }

    pub const fn clut_count(&self) -> usize {
        if self.clut_len != 0 {
            self.clut_data.len() / self.clut_len
        }
        else {
            0
        }
    }

    /// Byte length of a single CLUT.
    pub const fn clut_len(&self) -> usize {
        self.clut_len
    }

    /// Number of colors in the CLUTs.
    pub const fn clut_color_count(&self) -> usize {
        if let Some(bytes_per_element) = self.metadata.clut_type.bytes_per_element() {
            self.clut_len / bytes_per_element
        }
        else {
            0
        }
    }

    /// Get an image decoder for the image at the specified mipmap level,
    /// with mipmap level 0 being the base image.
    /// 
    /// If the image type is indexed and `clut` is `None`, the first CLUT in the picture will be used if it
    /// has one. Otherwise, [`Error::NoClutForIndexedImage`] will be returned.
    /// 
    /// If the image uses direct colors, `clut` is ignored. As a default, using `None` will work for
    /// all valid pictures.
    pub fn image<'a, 'b: 'a>(
        &'b self,
        mipmap: u8,
        clut: impl Into<Option<Clut<'a>>>
    ) -> Option<Result<ImageDecoder<'a, 'b>, Error>> {
        self.mipmaps.get(mipmap as usize)
            .map(|data| ImageDecoder::from_picture(self, clut, mipmap, data))
    }

    /// Get an iterator of image decoders for all mipmap levels.
    /// 
    /// If the image type is indexed and `clut` is `None`, the first CLUT in the picture will be used if it
    /// has one. Otherwise, [`Error::NoClutForIndexedImage`] will be returned.
    /// 
    /// If the image uses direct colors, `clut` is ignored. As a default, using `None` will work for
    /// all valid pictures.
    pub fn images<'a, 'b: 'a>(
        &'b self,
        clut: impl Into<Option<Clut<'a>>>
    ) -> Result<impl Iterator<Item = ImageDecoder<'a, 'b>>, Error> {
        let clut = self.metadata.image_type.is_indexed()
            .then(|| clut.into()
                .or_else(|| self.clut(0))
                .ok_or_else(|| Error::NoClutForIndexedImage)
            )
            .transpose()?;

        Ok(self.mipmaps.iter()
            .enumerate()
            .map(move |(i, data)|
                ImageDecoder::from_picture(self, clut.clone(), i as u8, data)
                    .unwrap() // OK: we've ensured that there's a CLUT
            )
        )
    }
}

/// Image data decoder.
pub struct ImageDecoder<'a, 'b> {
    width: u16,
    height: u16,
    pixel_decoder: PixelDecoder<'a>,
    data: &'b [u8],
}

impl<'a, 'b> ImageDecoder<'a, 'b> {
    /// Create a new image decoder.
    pub const fn new(
        width: u16,
        height: u16,
        pixel_decoder: PixelDecoder<'a>,
        data: &'b [u8]
    ) -> Self {
        Self { width, height, pixel_decoder, data }
    }

    /// Create a new image decoder with parameters derived from the picture. The mipmap level will determine
    /// the image's width and height.
    /// 
    /// If the image type is indexed and `clut` is `None`, the first CLUT in the picture will be used if it
    /// has one. Otherwise, [`Error::NoClutForIndexedImage`] will be returned.
    /// 
    /// If the image uses direct colors, `clut` is ignored. As a default, using `None` will work for
    /// all valid pictures.
    pub fn from_picture(
        picture: &'a Picture,
        clut: impl Into<Option<Clut<'a>>>,
        mipmap_level: u8,
        data: &'b [u8]
    ) -> Result<Self, Error> {
        let image_type = picture.metadata.image_type;
        Ok(ImageDecoder {
            width: picture.metadata.image_width >> mipmap_level,
            height: picture.metadata.image_height >> mipmap_level,
            pixel_decoder: match image_type {
                ImageType::Rgb16 | ImageType::Rgb24 | ImageType::Rgb32 => {
                    PixelDecoder::Direct(image_type.try_into().unwrap()) // always succeeds
                },

                ImageType::IdTex4 | ImageType::IdTex8 => {
                    PixelDecoder::Indexed {
                        codec: image_type.try_into().unwrap(),
                        clut: clut.into()
                            .or_else(|| picture.clut(0))
                            .ok_or_else(|| Error::NoClutForIndexedImage)?
                    }
                },
            },
            data
        })
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub const fn pixel_decoder(&self) -> &PixelDecoder<'a> {
        &self.pixel_decoder
    }

    /// Raw image data.
    pub const fn data(&self) -> &'b [u8] {
        self.data
    }

    /// Get the pixel at the specified position.
    pub fn pixel_at(&self, x: u16, y: u16) -> Option<Color> {
        if x < self.width && y < self.height {
            self.pixel(y as usize * self.width as usize + x as usize)
        }
        else {
            None
        }
    }

    /// Get the pixel at the specified index.
    pub fn pixel(&self, index: usize) -> Option<Color> {
        self.pixel_decoder.get(self.data, index)
    }

    /// Pixel count = width * height.
    pub const fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Read all of the pixels in the image.
    /// 
    /// Returns [`Error::InvalidImageParams`] if any of the pixels couldn't be read.
    pub fn read(&self) -> Result<Vec<Color>, Error> {
        (0..self.pixel_count())
            .map(|index| self.pixel(index))
            .collect::<Option<_>>()
            .ok_or_else(|| Error::InvalidImageParams)
    }

    /// Read all of the pixels in the image, converted to bytes in RGBA format.
    /// 
    /// Returns [`Error::InvalidImageParams`] if any of the pixels couldn't be read.
    /// 
    /// **Note:** The conversion uses the alpha value 128 (0x80) as fully opaque,
    /// as per the PS2 GS alpha value convention.
    pub fn read_to_rgba(&self) -> Result<Vec<u8>, Error> {
        let buffer_len = self.pixel_count() * 4;

        if let PixelDecoder::Direct(color_format) = self.pixel_decoder
            && color_format == ColorFormat::Rgb32
        {
            self.data.get(..buffer_len)
                .map(|s| s.to_vec())
                .ok_or_else(|| Error::InvalidImageParams)
        }
        else {
            let pixels = self.read()?;
            let mut buffer = Vec::with_capacity(buffer_len);
            for pixel in pixels {
                buffer.extend_from_slice(&pixel.to_rgba());
            }
            Ok(buffer)
        }
    }
}

/// Pixel data decoder. Interprets pixels as a [`Color`] in the specified format.
#[derive(Debug, Clone)]
pub enum PixelDecoder<'a> {
    Direct(ColorFormat),
    Indexed {
        codec: IndexCodec,
        clut: Clut<'a>
    }
}

impl<'a> PixelDecoder<'a> {
    /// Get the pixel at the specified index.
    pub fn get(&self, image_data: &[u8], index: usize) -> Option<Color> {
        match self {
            PixelDecoder::Direct(color_format) => {
                Color::from_pixel(image_data, index, *color_format)
            },

            PixelDecoder::Indexed { codec, clut } => {
                let color_index = codec.get(image_data, index)?;
                match codec {
                    IndexCodec::IdTex4 => clut.get_exact(color_index as _),
                    IndexCodec::IdTex8 => clut.get(color_index as _)
                }
            }
        }
    }
}

/// Color index codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndexCodec {
    IdTex4,
    IdTex8
}

impl IndexCodec {
    /// Get the color index at the specified index.
    pub fn get(&self, image_data: &[u8], index: usize) -> Option<u8> {
        match self {
            Self::IdTex4 => image_data.get(index / 2)
                .map(|b| if index % 2 == 0 { b & 0xF } else { b >> 4 }),

            Self::IdTex8 => image_data.get(index).copied()
        }
    }

    /// Encode indexes.
    pub fn encode<I>(&self, indexes: impl IntoIterator<Item = u8, IntoIter = I>) -> IndexEncoder<I> {
        IndexEncoder {
            codec: *self,
            indexes: indexes.into_iter()
        }
    }
}

impl TryFrom<ImageType> for IndexCodec {
    type Error = Error;

    fn try_from(value: ImageType) -> Result<Self, Self::Error> {
        Ok(match value {
            ImageType::IdTex4 => Self::IdTex4,
            ImageType::IdTex8 => Self::IdTex8,
            _ => return Err(Error::type_conversion::<ImageType, IndexCodec>(value))
        })
    }
}

/// Color index encoding iterator.
pub struct IndexEncoder<I> {
    codec: IndexCodec,
    indexes: I
}

impl<I: Iterator<Item = u8>> Iterator for IndexEncoder<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.codec {
            IndexCodec::IdTex4 => {
                let a = self.indexes.next()?;
                let b = self.indexes.next().unwrap_or(0);
                Some((a & 0xF) | (b & 0xF) << 4)
            },
            IndexCodec::IdTex8 => self.indexes.next()
        }
    }
}

/// Color lookup table (CLUT).
/// 
/// The data is stored as a `Cow`, which means it may be borrowed. Any mutation will
/// clone the data if it isn't already owned, therefore you cannot directly modify
/// the CLUT obtained from a [`Picture`].
/// 
/// Mutable operations will always assume a linear order, regardless if the CLUT is linear or not.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Clut<'a> {
    data: Cow<'a, [u8]>,
    color_format: ColorFormat,
    is_linear: bool
}

impl<'a> Clut<'a> {
    /// Create a new CLUT from borrowed data.
    /// 
    /// Data will be truncated to align with the color format's byte size.
    pub fn new_borrowed(data: &'a [u8], color_format: ColorFormat, is_linear: bool) -> Self {
        Self {
            data: Cow::Borrowed(&data[..data.len() - data.len() % color_format.size()]),
            color_format,
            is_linear
        }
    }

    /// Create a new CLUT with owned data.
    /// 
    /// Data will be truncated to align with the color format's byte size.
    pub fn new_owned(mut data: Vec<u8>, color_format: ColorFormat, is_linear: bool) -> Clut<'static> {
        data.truncate(data.len() - data.len() % color_format.size());
        Clut {
            data: Cow::Owned(data),
            color_format,
            is_linear
        }
    }

    /// Create a new CLUT with data borrowed from this CLUT.
    pub fn to_borrowed(&'a self) -> Self {
        Self::new_borrowed(&self.data, self.color_format, self.is_linear)
    }

    /// Convert this to an owned CLUT by cloning data.
    pub fn to_owned(&self) -> Clut<'static> {
        Self::new_owned(self.data.to_vec(), self.color_format, self.is_linear)
    }

    /// Move the data to an owned CLUT. Clones the data if it is not already owned.
    pub fn into_owned(self) -> Clut<'static> {
        Self::new_owned(self.data.into_owned(), self.color_format, self.is_linear)
    }

    /// Raw CLUT data.
    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Extracts the owned data.
    ///
    /// Clones the data if it is not already owned.
    pub fn into_owned_data(self) -> Vec<u8> {
        self.data.into_owned()
    }

    pub const fn color_format(&self) -> ColorFormat {
        self.color_format
    }

    pub const fn is_linear(&self) -> bool {
        self.is_linear
    }

    /// CLUT type for this CLUT's color format, with CSM flag.
    pub const fn clut_type(&self) -> ClutType {
        self.color_format.to_clut_type().union(
            if self.is_linear() { ClutType::CSM2 } else { ClutType::NONE }
        )
    }

    const fn swizzle_index(i: usize) -> usize {
        let j = (i ^ (i >> 1)) & 8;
        i ^ (j | (j << 1))
    }

    /// Get the color at the specified index.
    /// 
    /// If the CLUT is swizzled, this will return the color at the swizzled index.
    pub fn get(&self, i: usize) -> Option<Color> {
        self.get_exact(if self.is_linear() { i } else { Self::swizzle_index(i) })
    }

    /// Get the color at the exact index specified, no swizzling.
    pub fn get_exact(&self, i: usize) -> Option<Color> {
        Color::from_pixel(&self.data, i, self.color_format)
    }

    /// Number of colors in this CLUT.
    pub fn color_count(&self) -> usize {
        self.data.len() / self.color_format.size()
    }

    /// Get an iterator of all the colors in this CLUT.
    /// 
    /// The order of the colors will always be linear, regardless if the CLUT is linear or not.
    pub fn colors(&self) -> impl Iterator<Item = Color> {
        (0..self.color_count())
            .flat_map(|index| self.get_exact(index))
    }

    /// Appends a color to the end of a CLUT.
    /// 
    /// If the color's format does not match the CLUT's, it will be converted to the CLUT's color format.
    /// 
    /// Clones the data if it is not already owned.
    /// 
    /// ### Panics
    /// Panics if the new capacity exceeds `isize::MAX` *bytes*.
    pub fn push(&mut self, color: Color) {
        self.data.to_mut().extend_from_slice(&color.to_format(self.color_format));
    }

    /// Inserts a color at position `index` within the CLUT, shifting all colors after it to the right.
    /// 
    /// If the color's format does not match the CLUT's, it will be converted to the CLUT's color format.
    /// 
    /// Clones the data if it is not already owned.
    /// 
    /// ### Panics
    /// Panics if `index > color_count`.
    pub fn insert(&mut self, index: usize, color: Color) {
        let byte_index = index * self.color_format.size();
        self.data.to_mut().splice(byte_index..byte_index, color.to_format(self.color_format).iter().copied());
    }

    /// Removes the last color from the CLUT and returns it, or `None` if it is empty.
    /// 
    /// Clones the data if it is not already owned.
    pub fn pop(&mut self) -> Option<Color> {
        let color = self.get_exact(self.color_count() - 1)?;
        let data = self.data.to_mut();
        data.truncate(data.len() - color.len());
        Some(color)
    }

    /// Removes and returns the color at position index within the CLUT, shifting all colors after it to the left.
    /// 
    /// Clones the data if it is not already owned.
    /// 
    /// ### Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> Option<Color> {
        let color_size = self.color_format.size();
        let byte_index = index * color_size;
        let mut bytes = [0u8; 4]; // fits all color formats without allocating
        for (src, dest) in self.data.to_mut()
            .splice(byte_index..byte_index + color_size, [])
            .zip(bytes.iter_mut())
        {
            *dest = src;
        }

        Color::from_pixel(&bytes, 0, self.color_format)
    }

    /// Perform swizzling on the CLUT to an owned CLUT. The conversion works both ways,
    /// swizzling a linear CLUT and unswizzling a swizzled CLUT.
    /// 
    /// If you want to force a specific swizzling setting, see
    /// [`into_swizzled`](Self::into_swizzled) and [`into_linear`](Self::into_linear).
    pub fn swizzle(&self) -> Clut<'static> {
        let mut buffer = vec![0u8; self.data.len()];
        let color_size = self.color_format.size();
        for dest_index in 0..(buffer.len() / color_size) {
            let src_index = Self::swizzle_index(dest_index);
            let dest_offset = dest_index * color_size;
            let src_offset = src_index * color_size;
            for i in 0..color_size {
                buffer[dest_offset + i] = self.data[src_offset + i];
            }
        }

        Clut {
            data: Cow::Owned(buffer),
            color_format: self.color_format,
            is_linear: self.is_linear
        }
    }

    /// Convert this CLUT to a swizzled CLUT if it is linear. Otherwise, the CLUT is cloned.
    pub fn to_swizzled(&self) -> Clut<'static> {
        if self.is_linear { self.swizzle() } else { self.to_owned() }
    }

    /// Convert this CLUT to a linear CLUT if it is swizzled. Otherwise, the CLUT is cloned.
    pub fn to_linear(&self) -> Clut<'static> {
        if self.is_linear { self.to_owned() } else { self.swizzle() }
    }

    /// Convert this CLUT to a swizzled CLUT if it is linear. Otherwise, return it as an owned CLUT,
    /// cloning the data if it is not already owned.
    pub fn into_swizzled(self) -> Clut<'static> {
        if self.is_linear { self.swizzle() } else { self.into_owned() }
    }

    /// Convert this CLUT to a linear CLUT if it is swizzled. Otherwise, return it as an owned CLUT,
    /// cloning the data if it is not already owned.
    pub fn into_linear(self) -> Clut<'static> {
        if self.is_linear { self.into_owned() } else { self.swizzle() }
    }
}

impl<'a> Into<Clut<'a>> for &'a Clut<'_> {
    fn into(self) -> Clut<'a> {
        self.to_borrowed()
    }
}

impl<'a> Into<Option<Clut<'a>>> for &'a Clut<'_> {
    fn into(self) -> Option<Clut<'a>> {
        Some(self.to_borrowed())
    }
}

impl<'a> Into<Vec<u8>> for Clut<'a> {
    fn into(self) -> Vec<u8> {
        self.into_owned_data()
    }
}

/// Color representation in formats supported by TIM2 (in the image data or CLUT).
/// 
/// **Note:** The PS2 GS interprets alpha value 128 (0x80) as fully opaque.
/// Colors in TIM2 pictures should follow the same convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    Rgb16([u8; 2]),
    Rgb24([u8; 3]),
    Rgb32([u8; 4])
}

impl Color {
    /// Create a new color from a pixel in the image data at the specified index.
    pub fn from_pixel(data: &[u8], index: usize, format: ColorFormat) -> Option<Color> {
        Some(match format {
            ColorFormat::Rgb16 => Color::Rgb16(data.get(index * 2..(index + 1) * 2)?.try_into().unwrap()),
            ColorFormat::Rgb24 => Color::Rgb24(data.get(index * 3..(index + 1) * 3)?.try_into().unwrap()),
            ColorFormat::Rgb32 => Color::Rgb32(data.get(index * 4..(index + 1) * 4)?.try_into().unwrap())
        })
    }

    pub const fn format(&self) -> ColorFormat {
        match self {
            Color::Rgb16(_) => ColorFormat::Rgb16,
            Color::Rgb24(_) => ColorFormat::Rgb24,
            Color::Rgb32(_) => ColorFormat::Rgb32
        }
    }

    /// Convert this color to 8-bit RGBA.
    /// 
    /// **Note:** The resulting color keeps the alpha value as-is for Rgb32, and uses 128 (0x80)
    /// as the fully opaque value for other formats, as per the PS2 GS alpha value convention.
    pub const fn to_rgba(self) -> [u8; 4] {
        match self {
            Color::Rgb16(bytes) => {
                #[inline]
                const fn conv_5_to_8(x: u16) -> u8 {
                    ((x * 527 + 23) >> 6) as u8
                }

                let value = u16::from_le_bytes(bytes);
                [
                    conv_5_to_8(value & 0x1F),
                    conv_5_to_8(value >> 5 & 0x1F),
                    conv_5_to_8(value >> 10 & 0x1F),
                    (value >> 8 & 0x80) as u8
                ]
            },
            Color::Rgb24(b) => [b[0], b[1], b[2], 0x80],
            Color::Rgb32(b) => b
        }
    }

    /// Convert this color to the specified format.
    pub const fn to_format(self, format: ColorFormat) -> Color {
        // the things i do to stay const
        if self.format() as u8 == format as u8 {
            return self;
        }

        let rgba = self.to_rgba();
        Self::from_rgba(rgba, format)
    }

    /// Create a color from RGBA values, converted to the specified format.
    pub const fn from_rgba(rgba: [u8; 4], format: ColorFormat) -> Self {
        match format {
            ColorFormat::Rgb16 => {
                #[inline]
                const fn conv_8_to_5(x: u8) -> u16 {
                    (x as u16 * 249 + 1014) >> 11
                }

                let value = ((rgba[3] as u16) & 0x80) << 8
                    | conv_8_to_5(rgba[2]) << 10
                    | conv_8_to_5(rgba[1]) << 5
                    | conv_8_to_5(rgba[0]);

                Color::Rgb16(value.to_le_bytes())
            },
            ColorFormat::Rgb24 => Color::Rgb24([rgba[0], rgba[1], rgba[2]]),
            ColorFormat::Rgb32 => Color::Rgb32(rgba),
        }
    }
}

impl Deref for Color {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Color::Rgb16(bytes) => bytes,
            Color::Rgb24(bytes) => bytes,
            Color::Rgb32(bytes) => bytes
        }
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Color::Rgb16(bytes) => bytes,
            Color::Rgb24(bytes) => bytes,
            Color::Rgb32(bytes) => bytes
        }
    }
}

/// Color formats supported by TIM2 (in the image data or CLUT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorFormat {
    /// RGBA5551
    Rgb16,
    /// RGB888
    Rgb24,
    /// RGBA8888
    Rgb32
}

impl ColorFormat {
    /// Get the CLUT type corresponding to this color format (without CSM flag).
    pub const fn to_clut_type(&self) -> ClutType {
        match self {
            Self::Rgb16 => ClutType::RGB16,
            Self::Rgb24 => ClutType::RGB24,
            Self::Rgb32 => ClutType::RGB32
        }
    }

    /// Byte size for this color format.
    pub const fn size(&self) -> usize {
        match self {
            Self::Rgb16 => 2,
            Self::Rgb24 => 3,
            Self::Rgb32 => 4,
        }
    }
}

impl TryFrom<ClutType> for ColorFormat {
    type Error = Error;

    fn try_from(value: ClutType) -> Result<Self, Self::Error> {
        Ok(match value.color_type() {
            ClutType::RGB16 => ColorFormat::Rgb16,
            ClutType::RGB24 => ColorFormat::Rgb24,
            ClutType::RGB32 => ColorFormat::Rgb32,
            _ => return Err(Error::type_conversion::<ClutType, ColorFormat>(value))
        })
    }
}

impl Into<ClutType> for ColorFormat {
    fn into(self) -> ClutType {
        self.to_clut_type()
    }
}

impl TryFrom<ImageType> for ColorFormat {
    type Error = Error;

    fn try_from(value: ImageType) -> Result<Self, Self::Error> {
        Ok(match value {
            ImageType::Rgb16 => ColorFormat::Rgb16,
            ImageType::Rgb24 => ColorFormat::Rgb24,
            ImageType::Rgb32 => ColorFormat::Rgb32,
            _ => return Err(Error::type_conversion::<ImageType, ColorFormat>(value))
        })
    }
}

/// Mipmap info. Derived from the mipmap header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MipmapInfo {
    pub gs_miptbp1: u64,
    pub gs_miptbp2: u64
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clut_mutation() {
        let mut clut = Clut::new_owned(Vec::new(), ColorFormat::Rgb32, true);

        clut.push(Color::Rgb32([12, 34, 56, 128]));
        clut.push(Color::Rgb24([11, 22, 33]));
        clut.push(Color::Rgb24([44, 55, 66]));
        clut.push(Color::Rgb24([77, 88, 99]));

        assert_eq!(clut.get(0), Some(Color::Rgb32([12, 34, 56, 128])));
        assert_eq!(clut.get(1), Some(Color::Rgb32([11, 22, 33, 128])));
        assert_eq!(clut.get(2), Some(Color::Rgb32([44, 55, 66, 128])));
        assert_eq!(clut.pop(), Some(Color::Rgb32([77, 88, 99, 128])));
        assert_eq!(clut.remove(1), Some(Color::Rgb32([11, 22, 33, 128])));
        assert_eq!(clut.get(1), Some(Color::Rgb32([44, 55, 66, 128])));

        clut.insert(1, Color::Rgb24([67, 67, 67]));
        assert_eq!(clut.get(1), Some(Color::Rgb32([67, 67, 67, 128])));
        assert_eq!(clut.get(2), Some(Color::Rgb32([44, 55, 66, 128])));
    }
}