use std::borrow::Cow;

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

/// Picture info. Derived from the picture header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PictureInfo {
    /// Should always be 0.
    pub picture_format: u8,
    pub mipmap_count: u8,
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
    pub gs_texclut: u32
}

/// TIM2 picture. A single TIM2 file may contain multiple pictures.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Picture {
    info: PictureInfo,
    user_data: Vec<u8>,
    image_data: Vec<u8>,
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
    pub fn new(
        info: PictureInfo,
        user_data: Vec<u8>,
        image_data: Vec<u8>,
        clut_data: Vec<u8>,
        clut_color_count: u16
    ) -> Result<Self, Error> {
        let image_pixels = info.image_width as usize * info.image_height as usize;
        let image_data_len = info.image_type.bytes_per_n_pixels(image_pixels);
        if image_data.len() < image_data_len {
            return Err(Error::InvalidData("insufficient image data length"));
        }

        let clut_len = if info.clut_type != ClutType::NONE {
            let clut_bytes_per_element = info.clut_type.bytes_per_element()
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
            info,
            user_data,
            image_data,
            clut_data,
            clut_len
        })
    }

    pub const fn info(&self) -> PictureInfo {
        self.info
    }

    /// Raw user data. May contain an extended header.
    pub fn user_data(&self) -> &[u8] {
        &self.user_data
    }

    /// Raw image data.
    pub fn image_data(&self) -> &[u8] {
        &self.image_data
    }

    /// Get the CLUT at the specified index.
    pub fn clut(&self, index: usize) -> Option<Clut<'_>> {
        if self.clut_len == 0 { return None; }

        let offset = index * self.clut_len;
        self.clut_data
            .get(offset..offset + self.clut_len)
            .map(|data| Clut::new_borrowed(data, self.info.clut_type))
    }

    /// Get an iterator of all the CLUTs in this picture.
    pub fn cluts(&self) -> impl Iterator<Item = Clut<'_>> {
        std::iter::once(&self.clut_data)
            .filter(|_| self.clut_len != 0)
            .flat_map(|slice| slice.chunks_exact(self.clut_len))
            .map(|data| Clut::new_borrowed(data, self.info.clut_type))
    }

    /// Raw CLUT data.
    pub fn clut_data(&self) -> &[u8] {
        &self.clut_data
    }

    pub const fn clut_count(&self) -> usize {
        self.clut_data.len() / self.clut_len
    }

    /// Byte length of a single CLUT.
    pub const fn clut_len(&self) -> usize {
        self.clut_len
    }

    /// Number of colors in the CLUTs.
    pub const fn clut_color_count(&self) -> usize {
        if let Some(bytes_per_element) = self.info.clut_type.bytes_per_element() {
            self.clut_len / bytes_per_element
        }
        else {
            0
        }
    }

    /// Get an image reader for this picture's image.
    /// 
    /// If the image is indexed and `clut` is `None`, the first CLUT in the picture will be used if it
    /// has one. Otherwise, [`Error::NoClutForIndexedImage`] will be returned.
    /// 
    /// If the image uses direct colors, `clut` is ignored. As a default, using `None` will work for
    /// all valid pictures.
    pub fn image_reader<'a, 'b: 'a>(&'b self, clut: impl Into<Option<Clut<'a>>>) -> Result<ImageReader<'a, 'b>, Error> {
        let image_type = self.info.image_type;
        Ok(ImageReader {
            width: self.info.image_width,
            height: self.info.image_height,
            pixel_reader: match image_type {
                ImageType::Rgb16 | ImageType::Rgb24 | ImageType::Rgb32 => {
                    PixelReader::Direct(image_type.try_into().unwrap()) // always succeeds
                },

                ImageType::IdTex4 | ImageType::IdTex8 => {
                    PixelReader::Indexed {
                        four_bits: image_type == ImageType::IdTex4,
                        clut: clut.into()
                            .or_else(|| self.clut(0))
                            .ok_or_else(|| Error::NoClutForIndexedImage)?
                    }
                },
            },
            image_data: &self.image_data
        })
    }
}

/// Image data reader.
pub struct ImageReader<'a, 'b> {
    width: u16,
    height: u16,
    pixel_reader: PixelReader<'a>,
    image_data: &'b [u8],
}

impl<'a, 'b> ImageReader<'a, 'b> {
    /// Create a new image reader.
    pub const fn new(
        width: u16,
        height: u16,
        pixel_reader: PixelReader<'a>,
        image_data: &'b [u8]
    ) -> Self {
        Self { width, height, pixel_reader, image_data }
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub const fn pixel_reader(&self) -> &PixelReader<'a> {
        &self.pixel_reader
    }

    /// Raw image data.
    pub const fn image_data(&self) -> &'b [u8] {
        self.image_data
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
        self.pixel_reader.pixel(self.image_data, index)
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

    /// Read all of the pixels in the image, converted to RGBA format.
    /// 
    /// Returns [`Error::InvalidImageParams`] if any of the pixels couldn't be read.
    /// 
    /// **Note:** The conversion uses the alpha value 128 (0x80) as fully opaque,
    /// as per the PS2 GS alpha value convention.
    pub fn read_to_rgba(&self) -> Result<Vec<u8>, Error> {
        let buffer_len = self.pixel_count() * 4;

        if let PixelReader::Direct(color_format) = self.pixel_reader
            && color_format == ColorFormat::Rgb32
        {
            self.image_data.get(..buffer_len)
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

/// Pixel data reader. Interprets the pixel as a [`Color`] in the specified format.
#[derive(Debug, Clone)]
pub enum PixelReader<'a> {
    Direct(ColorFormat),
    Indexed {
        four_bits: bool,
        clut: Clut<'a>
    }
}

impl<'a> PixelReader<'a> {
    /// Get the pixel at the specified index.
    pub fn pixel(&self, image_data: &[u8], index: usize) -> Option<Color> {
        match self {
            PixelReader::Direct(color_format) => {
                Color::from_pixel(image_data, index, *color_format)
            },

            PixelReader::Indexed { four_bits, clut } => {
                if *four_bits {
                    let b = image_data.get(index / 2)?;
                    let color_index = if index % 2 == 0 { b & 0xF } else { b >> 4 };
                    clut.get_exact(color_index as _)
                }
                else {
                    let color_index = *image_data.get(index)?;
                    clut.get(color_index as _)
                }
            }
        }
    }
}

/// Color lookup table (CLUT).
/// 
/// The data may be borrowed (from a Picture).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Clut<'a> {
    data: Cow<'a, [u8]>,
    clut_type: ClutType
}

impl<'a> Clut<'a> {
    /// Create a new CLUT from borrowed data.
    pub const fn new_borrowed(data: &'a [u8], clut_type: ClutType) -> Self {
        Self {
            data: Cow::Borrowed(data),
            clut_type
        }
    }

    /// Create a new CLUT with data borrowed from this CLUT.
    pub fn to_borrowed(&'a self) -> Self {
        Self::new_borrowed(&self.data, self.clut_type)
    }

    /// Raw CLUT data.
    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub const fn clut_type(&self) -> ClutType {
        self.clut_type
    }

    /// Shorthand for `clut.clut_type().is_linear()`.
    pub const fn is_linear(&self) -> bool {
        self.clut_type.is_linear()
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
        Color::from_pixel(&self.data, i, self.clut_type.try_into().ok()?)
    }
}

impl Clut<'static> {
    /// Create a new CLUT with owned data.
    pub const fn new_owned(data: Vec<u8>, clut_type: ClutType) -> Self {
        Self {
            data: Cow::Owned(data),
            clut_type
        }
    }

    /// Create a new owned CLUT with data cloned from this CLUT.
    pub fn to_owned<'a>(&'a self) -> Self {
        Self::new_owned(self.data.to_vec(), self.clut_type)
    }

    /// Perform swizzling on the CLUT to an owned CLUT. The conversion works both ways,
    /// swizzling an unswizzled CLUT and unswizzling a swizzled CLUT.
    /// 
    /// Returns `None` if the CLUT type has no color format.
    pub fn swizzle<'a>(&'a self) -> Option<Self> {
        let mut buffer = vec![0u8; self.data.len()];
        let color_len = self.clut_type.bytes_per_element()?;
        for dest_index in 0..(buffer.len() / color_len) {
            let src_index = Self::swizzle_index(dest_index);
            let dest_offset = dest_index * color_len;
            let src_offset = src_index * color_len;
            for i in 0..color_len {
                buffer[dest_offset + i] = self.data[src_offset + i];
            }
        }

        Some(Self::new_owned(buffer, self.clut_type ^ ClutType::CSM2))
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