use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ParseError {
    MalformedOption(String),
    UnexpectedOption(String),
    MissingProgramName,
    MissingValue(String),
    BadInternalState,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MissingProgramName => {
                write!(f, "missing expected first argument (program name)")
            }
            ParseError::BadInternalState => {
                write!(f, "bad internal state, possibly bug in opts lib")
            }
            ParseError::MalformedOption(arg) => write!(f, "malformed option; got '{}'", arg),
            ParseError::UnexpectedOption(arg) => write!(f, "unexpected option; got '{}'", arg),
            ParseError::MissingValue(arg) => write!(f, "missinfg value for {}", arg),
        }
    }
}

impl Error for ParseError {}

#[derive(Debug)]
pub enum ValueError {
    WrongOptionType,
    ConversionError(String),
}

impl Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueError::WrongOptionType => write!(f, "wrong option type"),
            ValueError::ConversionError(val) => {
                write!(f, "error converting value '{}'", val)
            }
        }
    }
}

impl Error for ValueError {}
