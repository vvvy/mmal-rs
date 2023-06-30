use std::{fmt::{self, Debug, Display}, borrow::Cow};
use crate::ffi;

pub type MmalStatus = ffi::MMAL_STATUS_T::Type;

pub struct MmalError {
    status: Option<MmalStatus>,
    message: String
}

impl MmalError {
    pub(crate) fn with_status(message: String, status: MmalStatus) -> Self { Self { status: Some(status), message } }
    pub(crate) fn no_status(message: String) -> Self { Self { status: None, message } }

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


pub(crate) fn convert_status(status: MmalStatus, msg_f: impl FnOnce() -> String) -> Result<()> {
    if status == ffi::MMAL_STATUS_T::MMAL_SUCCESS {
        Ok(())
    } else {
        Err(MmalError::with_status(msg_f(), status).into())
    }
}

#[macro_export]
macro_rules! cst {
    ($status:expr, $s:literal) => {
        convert_status($status, || $s.to_owned())
    };
    ($status:expr, $fmt:literal, $($a:expr),+) => {
        convert_status($status, || format!($fmt, $($a),+))
    };
}

pub(crate) fn err_log_eligible() -> bool {
    std::env::var("MMAL_RS_ENABLE_LOG")
    .map(|s| if let Some(w) = s.parse::<u8>().ok() { 
        w != 0 
    } else {
        s == "true" || s == "yes"
    }).unwrap_or(false)
}

#[macro_export]
macro_rules! log_deinit {
    ($result:expr) => {
        if let Err(e) = $result {
            if *crate::error::LOG_ERRORS.get_or_init(err_log_eligible) {
                //TODO tracing or log
                eprintln!("deinit error: {}", e);
            }
        }
    };

}

pub(crate) const LOG_ERRORS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();