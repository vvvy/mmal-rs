use std::{fmt::{self, Debug, Display}, borrow::Cow};
use crate::ffi;

pub type MmalStatus = ffi::MMAL_STATUS_T::Type;

pub struct MmalError {
    status: Option<MmalStatus>,
    message: String
}

impl MmalError {
    pub fn with_status(message: String, status: MmalStatus) -> Self { Self { status: Some(status), message } }
    pub fn no_status(message: String) -> Self { Self { status: None, message } }

    pub fn message(&self) -> &str { &self.message }
    pub fn status_str(&self) -> Option<Cow<'static, str>> {
        unsafe {
            self.status.map(|s| std::ffi::CStr::from_ptr(ffi::mmal_status_to_string(s)).to_string_lossy())
        }
    }
}

impl Display for MmalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(status_str) = self.status_str() {
            write!(f, "[{}] {}", status_str, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl Debug for MmalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(status), Some(status_str)) = (self.status, self.status_str()) {
            write!(f, "[{}:{}] {}", status_str, status, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for MmalError { }

type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, MmalError>;

