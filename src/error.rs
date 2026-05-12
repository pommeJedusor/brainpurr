use std::{error::Error, fmt};

#[derive(Debug)]
pub enum BrainpurrError {
    TooManyNya,
    TooManyColonThree,
    ParsingError(String),
    CompilingError(String),
    UserInput(String),
    Output(String),
}

impl Error for BrainpurrError {}

impl fmt::Display for BrainpurrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TooManyNya => write!(f, "found a nya without its required :3"),
            Self::TooManyColonThree => write!(f, "found a :3 without its required nya"),
            Self::ParsingError(error_message) => {
                write!(f, "failed to parse the file: {error_message}")
            }
            Self::CompilingError(error_message) => {
                write!(f, "failed to compile the file: {error_message}")
            }
            Self::UserInput(error_message) => {
                write!(f, "failed to read user input: {error_message}")
            }
            Self::Output(error_message) => {
                write!(f, "failed to output: {error_message}")
            }
        }
    }
}
