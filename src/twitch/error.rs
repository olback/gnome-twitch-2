use std::fmt;

macro_rules! impl_from {
    ($type:ty) => {
        impl TwErrorTrait for $type {}
        impl From<$type> for TwError {
            fn from(e: $type) -> Self {
                Self {
                    line: line!(),
                    file: file!(),
                    error: Some(Box::new(e))
                }
            }
        }
    };
}


pub type TwResult<T> = std::result::Result<T, TwError>;
pub trait TwErrorTrait: fmt::Display + fmt::Debug + Send {}

#[derive(Debug)]
pub struct TwError {
    pub line: u32,
    pub file: &'static str,
    pub error: Option<Box<dyn TwErrorTrait>>
}

impl TwError {

    pub fn new(error: impl TwErrorTrait, line: u32, file: &'static str) -> Self {
        Self {
            line,
            file,
            error: Some(Box::new(format!("{}", error)))
        }
    }

    pub fn format(&self, location: bool) -> String {
        if location {
            format!("{}#{}: {}", self.file, self.line, self.format(false))
        } else {
            match &self.error {
                Some(e) => format!("{}", e),
                None => "Unknown Error".into()
            }
        }
    }

}

impl fmt::Display for TwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if crate::is_debug!() {
            write!(f, "{}", self.format(true))
        } else {
            write!(f, "{}", self.format(false))
        }
    }
}

impl_from!(std::string::String);
impl_from!(reqwest::Error);
impl_from!(reqwest::header::InvalidHeaderValue);
impl_from!(url::ParseError);
