use std::{fmt::{self, Debug, Display}, borrow::Cow};
use crate::ffi;

pub type MmalStatus = ffi::MMAL_STATUS_T::Type;

#[derive(Debug)]
pub enum Cause {
    Status(MmalStatus),
    CreatePool,
    CreateQueue,
    QueueEmpty,
    GetPort,
    InvalidEnumValue,
}

#[derive(Debug)]
pub struct MmalError {
    cause: Cause,
    message: String
}

impl MmalError {
    pub(crate) fn new(cause: Cause, message: String) -> Self { Self { cause, message } }
    pub(crate) fn with_status(status: MmalStatus, message: String) -> Self { Self { cause: Cause::Status(status), message } }
    //pub(crate) fn no_status(message: String) -> Self { Self { status: None, message } }
    pub(crate) fn with_cause(cause: Cause) -> Self { Self { cause, message: "".to_owned() } }

    pub fn message(&self) -> &str { &self.message }
    pub fn status_str(&self) -> Option<Cow<'static, str>> {
        unsafe {
            if let Cause::Status(s) = &self.cause {
                Some(std::ffi::CStr::from_ptr(ffi::mmal_status_to_string(*s)).to_string_lossy())
            } else {
                None
            }
        }
    }
}

impl Display for MmalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cause {
            Cause::Status(status) => write!(f, "[{}/{}]", self.status_str().unwrap_or(Cow::Borrowed("")), status)?, 
            Cause::CreatePool => write!(f, "(create pool)")?,
            Cause::CreateQueue => write!(f, "(create queue)")?,
            Cause::QueueEmpty => write!(f, "(queue empty)")?,
            Cause::GetPort => write!(f, "(get port)")?,
            Cause::InvalidEnumValue => write!(f, "(invalid enum value")?,
            
        }
        if !self.message.is_empty() {
            write!(f, ": {}", self.message())?
        }
        Ok(())

        /*
        if let Some(status_str) = self.status_str() {
            write!(f, "[{}] {}", status_str, self.message)
        } else {
            write!(f, "{}", self.message)
        }*/
    }
}


impl std::error::Error for MmalError { }

type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, MmalError>;


pub(crate) fn convert_status(status: MmalStatus, msg_f: impl FnOnce() -> String) -> Result<()> {
    if status == ffi::MMAL_STATUS_T::MMAL_SUCCESS {
        Ok(())
    } else {
        Err(MmalError::with_status(status, msg_f()).into())
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