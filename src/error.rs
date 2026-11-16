use std::{fmt::{Debug, Display}, num::TryFromIntError};

#[derive(Debug)]
pub enum Error {
    /// I/O error occured in the underlying reader/writer.
    IoError(std::io::Error),
    /// Invalid format signature (magic number).
    InvalidSignature,
    /// Unknown format ID in the TIM2 header.
    UnknownFormatId,
    /// Unimplemented feature.
    Unimplemented(&'static str),
    /// Invalid data.
    InvalidData(&'static str),
    /// The value does not represent a known image type.
    InvalidImageType(u8),
    /// The value does not represent a known CLUT type.
    InvalidClutType(u8),
    /// The value of the source type cannot be converted to any value of the destination type.
    TypeConversion {
        from: &'static str,
        to: &'static str,
        value: Box<dyn Debug + Send + Sync>
    },
    /// Cannot process an indexed image without a CLUT.
    NoClutForIndexedImage,
    /// The image parameters do not match the supplied data.
    InvalidImageParams,
    /// Attempted to write past the "end of file".
    EofExpected,
    TryFromIntError(TryFromIntError)
}

impl Error {
    pub(crate) fn type_conversion<From, To>(value: impl Debug + Send + Sync + 'static) -> Self
    where
        From: ?Sized,
        To: ?Sized
    {
        // lazy type name trimming
        fn type_name<T: ?Sized>() -> &'static str {
            let name = std::any::type_name::<T>();
            &name[name.rfind(':').map_or(0, |i| i + 1)..]
        }

        Self::TypeConversion {
            from: type_name::<From>(),
            to: type_name::<To>(),
            value: Box::new(value)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(error) => Display::fmt(&error, f),
            Error::InvalidSignature => write!(f, "Invalid signature"),
            Error::UnknownFormatId => write!(f, "Unknown format ID"),
            Error::Unimplemented(reason) => write!(f, "Unimplemented: {reason}"),
            Error::InvalidData(reason) => write!(f, "Invalid data: {reason}"),
            Error::InvalidImageType(value) => write!(f, "Invalid image type: {value}"),
            Error::InvalidClutType(value) => write!(f, "Invalid CLUT type: {value}"),
            Error::TypeConversion { from, to, value } => write!(f, "Cannot convert this {from} to {to}: {:?}", value),
            Error::NoClutForIndexedImage => write!(f, "Cannot process an indexed image without a CLUT"),
            Error::InvalidImageParams => write!(f, "Invalid image parameters"),
            Error::EofExpected => write!(f, "EOF expected"),
            Error::TryFromIntError(error) => Display::fmt(&error, f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self::TryFromIntError(value)
    }
}