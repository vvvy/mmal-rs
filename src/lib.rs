pub mod mmalcore;
pub mod param;
pub mod camera_info;
pub mod camera;
pub mod encoder;

use std::{mem, ffi::{CStr, c_char}, fmt::{self, Debug, Display}, borrow::Cow};

use mmal_sys as ffi;

type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, MmalError>;

pub use mmalcore::*;
pub use param::*;
pub use camera_info::*;
pub use camera::*;
pub use encoder::*;

unsafe fn fix_encoding(port: *mut ffi::MMAL_PORT_T, encoding: u32) -> u32 {
    // On firmware prior to June 2016, camera and video_splitter
    // had BGR24 and RGB24 support reversed.
    if encoding == ffi::MMAL_ENCODING_RGB24 || encoding == ffi::MMAL_ENCODING_BGR24 {
        if ffi::mmal_util_rgb_order_fixed(port) != 0 {
            ffi::MMAL_ENCODING_RGB24
        } else {
            ffi::MMAL_ENCODING_BGR24
        }
    } else {
        encoding
    }
}

pub type MmalStatus = ffi::MMAL_STATUS_T::Type;

fn cst(status: MmalStatus, msg_f: impl FnOnce() -> String) -> Result<()> {
    if status == ffi::MMAL_STATUS_T::MMAL_SUCCESS {
        Ok(())
    } else {
        Err(MmalError::with_status(msg_f(), status).into())
    }
}

fn msgf(message: &'static str, entity: &'static str) -> impl FnOnce() -> String {
    || message.to_owned() + " " + entity
}



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