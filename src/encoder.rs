use crate::idp;

use super::*;

//------------------------------------------------------------------------------------------------------------------------------

pub struct EncoderEntity;

impl Entity for EncoderEntity {
    fn name() -> &'static str { "encoder" }
}

impl ComponentEntity for EncoderEntity { }

pub type EncoderComponentHandle = ComponentHandle<EncoderEntity>;

impl EncoderComponentHandle {
    pub fn create() -> Result<Self> {
        let component_name: *const c_char = ffi::MMAL_COMPONENT_DEFAULT_IMAGE_ENCODER.as_ptr() as *const c_char;
        unsafe {
            Self::create_from(component_name)
        }
    }

    //unsafe fn input_port(&self)-> *mut ffi::MMAL_PORT_T { self.input_port_n(0) }
    //unsafe fn output_port(&self)-> *mut ffi::MMAL_PORT_T { self.output_port_n(0) }
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct EncoderInputPort;
impl ComponentPort for EncoderInputPort {
    type E = EncoderEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.input_port_n(DEFAULT_PORT_OFFSET)
    }

    fn name() -> &'static str { "encoder input port" }
}

pub struct EncoderOutputPort;
impl ComponentPort for EncoderOutputPort {
    type E = EncoderEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.output_port_n(DEFAULT_PORT_OFFSET)
    }

    fn name() -> &'static str { "encoder output port" }
}

//------------------------------------------------------------------------------------------------------------------------------

idp!{MMAL_PARAMETER_JPEG_Q_FACTOR}
///JPEG Quality Factor (default 90)
pub type JpegQFactor = Param<EncoderOutputPort, Uint32<MMAL_PARAMETER_JPEG_Q_FACTOR>>;

idp!{MMAL_PARAMETER_JPEG_RESTART_INTERVAL}
/// JPEG Restart interval (default 0)
pub type JpegRestartInterval = Param<EncoderOutputPort, Uint32<MMAL_PARAMETER_JPEG_RESTART_INTERVAL>>;


//------------------------------------------------------------------------------------------------------------------------------
pub struct EncoderOutFormat {
    encoding: u32,
}

impl Default for EncoderOutFormat {
    fn default() -> Self { Self { 
        encoding: ffi::MMAL_ENCODING_JPEG
    } }
}


impl PortConfig for EncoderOutFormat {
    unsafe fn apply_format(&self, port: *mut ffi::MMAL_PORT_T) {
        let format = &mut (*(*port).format);
        format.encoding = self.encoding;
    }

    unsafe fn apply_buffer_policy(&self, port: *mut ffi::MMAL_PORT_T) {
        let port = &mut *port;

        port.buffer_num = port.buffer_num_recommended;
        if port.buffer_num < port.buffer_num_min { 
            port.buffer_num = port.buffer_num_min;
        }

        port.buffer_size = port.buffer_size_recommended;
        if port.buffer_size < port.buffer_size_min { 
            port.buffer_size = port.buffer_size_min;
        }
    }
}
