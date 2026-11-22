//! # tim2-rs
//! 
//! Rust decoding and encoding library for TIM2, an image format that was widely used on the PlayStation 2
//! for storing texture images.
//! 
//! # Decoding
//! 
//! To decode a TIM2 file, first create a [`Decoder`] from a reader, then use [`Decoder::next_picture()`]
//! to iterate through the pictures:
//! 
//! ```ignore
//! use std::{fs::File, io::BufReader};
//! use tim2::Decoder;
//! 
//! let file = File::open(path)?;
//! let mut decoder = Decoder::from_reader(BufReader::new(file))?;
//! 
//! while let Some(picture) = decoder.next_picture()? {
//!    // do something with the picture...
//! }
//! ```
//! 
//! You can decode the picture's image by obtaining an [`ImageDecoder`] using [`Picture::image()`].
//! You'll need to specify a mipmap level and a CLUT to use for indexed images. The CLUT can be `None`
//! to use the first CLUT in the picture. For direct color images, it is ignored and should be `None`.
//! 
//! ```ignore
//! let mipmap_level: u8 = 0; // Mipmap level (0-6). LV0 = base image
//! let clut: Option<Clut> = None; // CLUT
//! 
//! // Optionally, obtain a different CLUT from the picture
//! let clut = picture.clut(1);
//! 
//! if let Some(result) = picture.image(mipmap_level, clut) {
//!     let image: ImageDecoder = result?;
//!     // use the image decoder...
//! }
//! ```
//! 
//! The image decoder can be used in a number of ways. You can read the image pixel by pixel:
//! 
//! ```ignore
//! let pixel: Option<Color> = image.pixel_at(12, 34); // x, y
//! // Or you can use a direct index
//! let pixel: Option<Color> = image.pixel(123);
//! ```
//! 
//! You can also read all of the pixels at once:
//! 
//! ```ignore
//! let pixels: Vec<Color> = image.read()?;
//! ```
//! 
//! For convenience, you can also read it to an RGBA buffer:
//! 
//! ```ignore
//! let pixels_data: Vec<u8> = image.read_to_rgba()?;
//! ```
//! 
//! # Encoding
//! 
//! To encode a TIM2 file, first create an [`Encoder`] from a writer, then call [`Encoder::write_picture()`]
//! as many times as the number of pictures to finish writing the file:
//! 
//! ```ignore
//! use std::{fs::File, io::BufWriter};
//! use tim2::{Encoder, Picture, Info, FormatId};
//! 
//! let pictures: Vec<Picture>;
//! let info = Info {
//!     is_clut2: false,
//!     format_version: 4, // should be 3 or 4
//!     format_id: FormatId::Align128,
//!     picture_count: pictures.len() as u16
//! };
//! 
//! let file = File::create(path)?;
//! let mut encoder = Encoder::from_writer(BufWriter::new(file), &info)?;
//! 
//! for picture in pictures.iter() {
//!     encoder.write_picture(picture)?;
//! }
//! ```
//! 
//! # Creating a [`Picture`]
//! 
//! To create a [`Picture`] from scratch, you'll need the metadata, the image data and the CLUT data
//! (if the image is indexed).
//! 
//! The image data or CLUT data is simply a byte sequence of colors. This library does not include
//! an "ImageEncoder", but you can very easily encode them yourself, using the [`Color`] struct
//! to perform format conversion if needed.
//! 
//! ```
//! use tim2::{Color, ColorFormat};
//! 
//! let mut rgba: Vec<u8> = Vec::new();
//! // Add a red pixel to the RGBA data (note the 128 alpha value; this is fully opaque to the PS2 GS)
//! rgba.extend(&[255, 0, 0, 128]);
//! 
//! let mut rgb16: Vec<u8> = Vec::new();
//! // Add a green pixel to the RGB16 (RGBA5551) data by converting from RGB24 (RGB888)
//! rgb16.extend(&*Color::Rgb24([0, 255, 0]).to_format(ColorFormat::Rgb16)); // Color derefs to &[u8]
//! ```
//! 
//! Once you have the data required, you can create a new [`Picture`] using [`Picture::new()`].
//! If you only have one base image, you can simply pass `vec![image_data]` to the `mipmaps` parameter.
//! Otherwise, each element in `mipmaps` is the image data for a mipmap level, from level 0
//! (base image) to level 6, in that order. There must not be more than 7 mipmaps.
//! 
//! Note that each mipmap must be half the size of the previous one, otherwise the encoded picture
//! will be gibberish.

mod common;
mod decoder;
mod encoder;
mod error;
mod internal;

pub use common::*;
pub use decoder::*;
pub use encoder::*;
pub use error::*;