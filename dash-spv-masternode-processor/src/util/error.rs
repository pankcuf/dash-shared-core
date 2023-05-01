use std::fmt::{Debug, Display, Formatter};
// use url;

#[derive(Debug, Hash)]
pub enum Error {
    Default(String),
    DefaultWithCode(String, u32)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Default(message) => Debug::fmt(&message, f),
            Error::DefaultWithCode(message, code) => Debug::fmt(&format!("{:?}: {}", message, code), f)
        }
    }
}

impl Error {
    pub fn code(&self) -> u32 {
        match self {
            Error::Default(..) => 0,
            Error::DefaultWithCode(_, code) => *code,
        }
    }
    pub fn message(&self) -> &String {
        match self {
            Error::Default(message) => message,
            Error::DefaultWithCode(message, _) => message,
        }
    }
}

impl std::error::Error for Error {

}

// impl From<url::ParseError> for Error {
//     fn from(e: url::ParseError) -> Error {
//         Error::Default(e.to_owned().to_string())
//     }
// }
