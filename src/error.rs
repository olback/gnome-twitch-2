use std::fmt;

#[macro_export]
macro_rules! p {
    ($e:expr) => {
        match $e {
            Ok(v) => v,

            Err(e) => return Err(GtError {
                line: line!(),
                file: file!(),
                error: Some(Box::new(e))
            })
        }
    };
}

macro_rules! impl_from {
    ($type:ty) => {
        impl GtErrorTrait for $type {}
        // impl From<$type> for GtError {
        //     fn from(e: $type) -> Self {
        //         Self {
        //             line: line!(),
        //             file: file!(),
        //             error: Some(Box::new(e))
        //         }
        //     }
        // }
    };
}

pub trait GtErrorTrait: fmt::Display + fmt::Debug {}

#[derive(Debug)]
pub struct GtError {
    pub line: u32,
    pub file: &'static str,
    pub error: Option<Box<dyn GtErrorTrait>>
}

impl GtError {

    pub fn new(error: impl GtErrorTrait, line: u32, file: &'static str) -> Self {
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

impl fmt::Display for GtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if crate::is_debug!() {
            write!(f, "{}", self.format(true))
        } else {
            write!(f, "{}", self.format(false))
        }
    }
}

impl_from!(std::io::Error);
impl_from!(std::string::String);
