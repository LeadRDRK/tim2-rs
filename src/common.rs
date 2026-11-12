use std::borrow::Cow;

use bitflags::bitflags;

use crate::{Error, internal::PictureHeader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    pub version: u8,
    pub format: u8,
    pub picture_count: u16
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ClutType: u8 {
        const RGB16 = 0b00000001;
        const RGB32 = 0b00000011;
        const CSM2  = 0b10000000;

        /// PAL_NONE (no palette)
        const NONE = 0;
        /// PAL_RGB16_CSM1 (16-bit palette, swizzled)
        const RGB16_CSM1 = Self::RGB16.bits();
        /// PAL_RGB32_CSM1 (32-bit palette, swizzled)
        const RGB32_CSM1 = Self::RGB32.bits();
        /// PAL_RGB16_CSM2 (16-bit palette, linear)
        const RGB16_CSM2 = Self::RGB16.bits() | Self::CSM2.bits();
        /// PAL_RGB32_CSM2 (32-bit palette, linear)
        const RGB32_CSM2 = Self::RGB32.bits() | Self::CSM2.bits();

        // allow any other bit to be set to handle invalid values
        const _ = !0;
    }
}

impl ClutType {
    pub const fn color_type(self) -> ClutType {
        self.difference(Self::CSM2)
    }

    pub const fn is_linear(&self) -> bool {
        self.contains(Self::CSM2)
    }

    pub const fn bits_per_element(&self) -> Option<usize> {
        Some(match self.color_type() {
            ClutType::RGB16 => 16,
            ClutType::RGB32 => 32,
            _ => return None
        })
    }

    pub const fn bytes_per_element(&self) -> Option<usize> {
        Some(match self.color_type() {
            ClutType::RGB16 => 2,
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
            3 => Self::RGB32_CSM1,
            129 => Self::RGB16_CSM2,
            131 => Self::RGB32_CSM2,
            _ => return Err(Error::InvalidClutType(value))
        })
    }
}

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
    pub fn bits_per_pixel(&self) -> usize {
        match self {
            ImageType::Rgb16  => 16,
            ImageType::Rgb24  => 24,
            ImageType::Rgb32  => 32,
            ImageType::IdTex4 => 4,
            ImageType::IdTex8 => 8,
        }
    }

    pub fn bytes_per_n_pixels(&self, n: usize) -> usize {
        match self {
            ImageType::Rgb16  => n * 2,
            ImageType::Rgb24  => n * 3,
            ImageType::Rgb32  => n * 4,
            ImageType::IdTex4 => n.div_ceil(2),
            ImageType::IdTex8 => n,
        }
    }

    pub fn is_indexed(&self) -> bool {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PictureInfo {
    pub picture_format: u8,
    pub mipmap_count: u8,
    pub clut_type: ClutType,
    pub image_type: ImageType,
    pub image_width: u16,
    pub image_height: u16,
    pub gs_tex0: u64,
    pub gs_tex1: u64,
    pub gs_texafbapabe: u32,
    pub gs_texclut: u32
}

impl TryFrom<PictureHeader> for PictureInfo {
    type Error = Error;

    fn try_from(header: PictureHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            picture_format: header.picture_format,
            mipmap_count: header.mipmap_count,
            clut_type: header.clut_type.try_into()?,
            image_type: header.image_type.try_into()?,
            image_width: header.image_width.get(),
            image_height: header.image_height.get(),
            gs_tex0: header.gs_tex0.get(),
            gs_tex1: header.gs_tex1.get(),
            gs_texafbapabe: header.gs_texafbapabe.get(),
            gs_texclut: header.gs_texclut.get()
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Picture {
    info: PictureInfo,
    image_data: Vec<u8>,
    cluts: Vec<Vec<u8>>
}

impl Picture {
    pub fn from_data(
        info: PictureInfo,
        image_data: Vec<u8>,
        cluts: Vec<Vec<u8>>
    ) -> Result<Self, Error> {
        let image_pixels = info.image_width as usize * info.image_height as usize;
        let image_data_len = info.image_type.bytes_per_n_pixels(image_pixels);
        if image_data.len() < image_data_len { // trailing data is valid
            return Err(Error::InvalidData("insufficient image data length"));
        }

        if info.clut_type != ClutType::NONE {
            let clut_bytes_per_element = info.clut_type.bytes_per_element()
                .ok_or_else(|| Error::InvalidData("unknown CLUT color type"))?;

            if cluts.iter().any(|clut| clut.len() % clut_bytes_per_element != 0) {
                return Err(Error::InvalidData("unexpected CLUT data length"));
            }
        }

        Ok(Self {
            info,
            image_data,
            cluts
        })
    }

    pub fn info(&self) -> PictureInfo {
        self.info
    }

    pub fn image_data(&self) -> &[u8] {
        &self.image_data
    }

    pub fn clut(&self, index: usize) -> Option<Clut<'_>> {
        self.cluts
            .get(index)
            .map(|data| Clut::new_borrowed(data, self.info.clut_type))
    }

    pub fn cluts(&self) -> impl Iterator<Item = Clut<'_>> {
        self.cluts
            .iter()
            .map(|data| Clut::new_borrowed(data, self.info.clut_type))
    }

    pub fn clut_buffers(&self) -> &[Vec<u8>] {
        &self.cluts
    }

    pub fn clut_count(&self) -> usize {
        self.cluts.len()
    }

    /// Get an image reader for this picture's image.
    /// 
    /// If the image is indexed and `clut` is `None`, the first CLUT in the picture will be used if it
    /// has one. Otherwise, `Error::NoClutForIndexedImage` will be returned. \
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

pub struct ImageReader<'a, 'b> {
    width: u16,
    height: u16,
    pixel_reader: PixelReader<'a>,
    image_data: &'b [u8],
}

impl<'a, 'b> ImageReader<'a, 'b> {
    pub fn new(
        width: u16,
        height: u16,
        pixel_reader: PixelReader<'a>,
        image_data: &'b [u8]
    ) -> Self {
        Self { width, height, pixel_reader, image_data }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn pixel_reader(&self) -> &PixelReader<'a> {
        &self.pixel_reader
    }

    pub fn image_data(&self) -> &'b [u8] {
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

    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Read all of the pixels in the image.
    /// 
    /// Returns `Error::InvalidImageParams` if any of the pixels couldn't be read.
    pub fn read(&self) -> Result<Vec<Color>, Error> {
        (0..self.pixel_count())
            .map(|index| self.pixel(index))
            .collect::<Option<_>>()
            .ok_or_else(|| Error::InvalidImageParams)
    }

    /// Read all of the pixels in the image, converted to RGBA format.
    /// 
    /// Returns `Error::InvalidImageParams` if any of the pixels couldn't be read.
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
                let color_index = if *four_bits {
                    let b = image_data.get(index / 2)?;
                    if index % 2 == 0 { b & 0xF } else { b >> 4 }
                }
                else {
                    *image_data.get(index)?
                };

                clut.get(color_index as _)
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Clut<'a> {
    data: Cow<'a, [u8]>,
    clut_type: ClutType
}

impl<'a> Clut<'a> {
    pub fn new_borrowed(data: &'a [u8], clut_type: ClutType) -> Self {
        Self {
            data: data.into(),
            clut_type
        }
    }

    pub fn to_borrowed(&'a self) -> Self {
        Self::new_borrowed(&self.data, self.clut_type)
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn clut_type(&self) -> ClutType {
        self.clut_type
    }

    pub fn is_linear(&self) -> bool {
        self.clut_type.is_linear()
    }

    fn swizzle_index(i: usize) -> usize {
        let j = (i ^ (i >> 1)) & 8;
        i ^ (j | (j << 1))
    }

    pub fn get(&self, i: usize) -> Option<Color> {
        let i = if self.is_linear() { i } else { Self::swizzle_index(i) };
        Color::from_pixel(&self.data, i, self.clut_type.try_into().ok()?)
    }
}

impl Clut<'static> {
    pub fn new_owned(data: Vec<u8>, clut_type: ClutType) -> Self {
        Self {
            data: data.into(),
            clut_type
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    Rgb16([u8; 2]),
    Rgb24([u8; 3]),
    Rgb32([u8; 4])
}

impl Color {
    pub fn from_pixel(data: &[u8], index: usize, format: ColorFormat) -> Option<Color> {
        Some(match format {
            ColorFormat::Rgb16 => Color::Rgb16(data.get(index * 2..(index + 1) * 2)?.try_into().unwrap()),
            ColorFormat::Rgb24 => Color::Rgb24(data.get(index * 3..(index + 1) * 3)?.try_into().unwrap()),
            ColorFormat::Rgb32 => Color::Rgb32(data.get(index * 4..(index + 1) * 4)?.try_into().unwrap())
        })
    }

    pub fn format(&self) -> ColorFormat {
        match self {
            Color::Rgb16(_) => ColorFormat::Rgb16,
            Color::Rgb24(_) => ColorFormat::Rgb24,
            Color::Rgb32(_) => ColorFormat::Rgb32
        }
    }

    pub fn to_rgba(self) -> [u8; 4] {
        match self {
            Color::Rgb16(bytes) => {
                #[inline]
                fn conv_5_to_8(x: u16) -> u8 {
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

    pub fn to_format(self, format: ColorFormat) -> Color {
        if self.format() == format {
            return self;
        }

        let rgba = self.to_rgba();
        match format {
            ColorFormat::Rgb16 => {
                #[inline]
                fn conv_8_to_5(x: u8) -> u16 {
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