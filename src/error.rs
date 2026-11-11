use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    InvalidSignature,
    UnknownFormatId,
    Unimplemented(&'static str),
    InvalidData(&'static str),
    InvalidImageType(u8),
    InvalidClutType(u8),
    TypeConversion {
        from: &'static str,
        to: &'static str,
        value: Box<dyn Debug>
    },
    NoClutForIndexedImage,
    InvalidImageParams
}

impl Error {
    pub(crate) fn type_conversion<From, To>(value: impl Debug + 'static) -> Self
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
            Error::NoClutForIndexedImage => write!(f, "Cannot decode indexed image without a CLUT"),
            Error::InvalidImageParams => write!(f, "Invalid image parameters")
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}