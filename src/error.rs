use std::{error, fmt, io, num};

#[derive(Debug)]
pub enum Error {
    TooManyOpenLoop,
    TooManyCloseLoop,
    PointerOverflow,
    PointerUnderflow,
    IO(io::Error),
    UserInputParsing(num::ParseIntError),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TooManyOpenLoop => write!(f, "found more 'loop opening' than 'loop closing'"),
            Self::TooManyCloseLoop => write!(f, "found more 'loop closing' than 'loop opening'"),
            Self::PointerUnderflow => write!(f, "Pointer Overflow"),
            Self::PointerOverflow => write!(f, "Pointer Underflow"),
            Self::IO(error) => {
                write!(f, "failed to output: {}", error)
            }
            Self::UserInputParsing(error) => {
                write!(f, "failed to parse input: {}", error)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Error::UserInputParsing(error)
    }
}
