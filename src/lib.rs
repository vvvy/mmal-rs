mod error;
pub mod mmalcore;
pub mod param;
pub mod camera_info;
pub mod camera;
pub mod encoder;
pub mod video_encoder;
pub mod ffi;

use std::{mem, ffi::{CStr, c_char}, fmt::Debug};

pub use error::*;
pub use mmalcore::*;
pub use param::*;
pub use camera_info::*;
pub use camera::*;
pub use encoder::*;
pub use video_encoder::*;

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



