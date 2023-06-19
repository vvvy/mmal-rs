use super::*;

#[derive(Clone, Debug, Default)]
pub struct CameraInstanceInfo {
    pub port_id: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub lens_present: bool,
    pub camera_name: String,
}

#[derive(Clone, Debug, Default)]
pub struct CameraInfo {
    pub cameras: Vec<CameraInstanceInfo>
}

//------------------------------------------------------------------------------------------------------------------------------


pub struct CameraInfoEntity;

impl Entity for CameraInfoEntity {
    fn name() -> &'static str { "camera-info" }
}

/* 
/// Camera settinggs
#[derive(Clone, Debug)]
pub enum CameraInfoParameter {
    /// Camera information
    CameraInfo(CameraInfo),
}

impl TryInto<(u32, ParamValue)> for CameraInfoParameter {
    type Error = MmalError;
    fn try_into(self) -> Result<(u32, ParamValue)> {
        match self {
            CameraInfoParameter::CameraInfo(v) => Ok((ffi::MMAL_PARAMETER_CAMERA_INFO, ParamValue::CameraInfo(v))),
        }
    }
}

impl ParamUpdate for CameraInfoParameter {
    fn update(&mut self, pv: ParamValue) {
        match (self, pv) {
            (CameraInfoParameter::CameraInfo(t), ParamValue::CameraInfo(v)) => *t = v,
            (a, b) => panic!("Unsuported parameter update: {a:?} <- {b:?}")
        }
    }
}

*/



impl ComponentEntity for CameraInfoEntity {
    //type InputPort = EmptyPortSet;
    //type OutputPort = EmptyPortSet;
    //type ComponentParam = CameraInfoParameter;
}

pub type CameraInfoComponentHandle = ComponentHandle<CameraInfoEntity>;

impl CameraInfoComponentHandle {
    pub fn create() -> Result<Self> {
        let component_name: *const c_char = ffi::MMAL_COMPONENT_DEFAULT_CAMERA_INFO.as_ptr() as *const c_char;
        unsafe { Self::create_from(component_name) }        
    }
}



pub struct CameraInfoControlPort;

impl ComponentPort for CameraInfoControlPort {
    type E = CameraInfoEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.control_port()
    }

    fn name() -> &'static str { "camera-info control port" }
}


pub struct CameraInfoInnerType {
    inner: ffi::MMAL_PARAMETER_CAMERA_INFO_T
}

impl From<&CameraInfoInnerType> for CameraInfo {
    fn from(value: &CameraInfoInnerType) -> Self {
        let cameras = unsafe { value.inner.cameras
            .into_iter()
            .take(value.inner.num_cameras as usize)
            .map(|w| CameraInstanceInfo {
                port_id: w.port_id,
                max_width: w.max_width,
                max_height: w.max_height,
                lens_present: w.lens_present != 0,
                camera_name: CStr::from_ptr(w.camera_name.as_ptr()).to_string_lossy().into_owned(),
            })
            .collect() };

        Self { cameras }
    }
}

impl Default for CameraInfoInnerType {
    fn default() -> Self {
        let inner = unsafe {
            let mut info: ffi::MMAL_PARAMETER_CAMERA_INFO_T = mem::zeroed();
            info.hdr.id = ffi::MMAL_PARAMETER_CAMERA_INFO as u32;
            info.hdr.size = mem::size_of::<ffi::MMAL_PARAMETER_CAMERA_INFO_T>() as u32;
            info
        };
        Self { inner }
    }
}

impl InnerParamType for CameraInfoInnerType {
    fn name() -> &'static str {"CameraInfo" }

    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> u32 {
        ffi::mmal_port_parameter_get(port, &mut self.inner.hdr)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> u32 {
        ffi::mmal_port_parameter_set(port, &self.inner.hdr)
    }
}

pub type CameraInformation = Param<CameraInfoControlPort, CameraInfoInnerType>;


