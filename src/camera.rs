use super::*;
use crate::{idp, enumize, enumerated_inner_type};



pub struct CameraEntity;

impl Entity for CameraEntity {
    fn name() -> &'static str { "camera" }
}


//------------------------------------------------------------------------------------------------------------------------------

impl ComponentEntity for CameraEntity { }

pub type CameraComponentHandle = ComponentHandle<CameraEntity>;

impl CameraComponentHandle {
    pub fn create() -> Result<Self> {
        let component_name: *const c_char = ffi::MMAL_COMPONENT_DEFAULT_CAMERA.as_ptr() as *const c_char;
        unsafe { Self::create_from(component_name) }        
    }

    //unsafe fn capture_port(&self) -> *mut ffi::MMAL_PORT_T { self.output_port_n(CameraOutputPort::Capture as isize) }
    //unsafe fn preview_port(&self) -> *mut ffi::MMAL_PORT_T { self.output_port_n(CameraOutputPort::Preview as isize) }
    //unsafe fn video_port(&self) -> *mut ffi::MMAL_PORT_T { self.output_port_n(CameraOutputPort::Video as isize) }

    /* 
    pub fn configure_capture_port(&self, config: impl PortConfig) -> Result<()> {
        unsafe { CameraEntity::configure_output_port(self, CameraOutputPort::Capture, config) }
    }
    pub fn configure_preview_port(&self, config: impl PortConfig) -> Result<()> {
        unsafe { CameraEntity::configure_output_port(self, CameraOutputPort::Preview, config) }
    }
    pub fn configure_video_port(&self, config: impl PortConfig) -> Result<()> {
        unsafe { CameraEntity::configure_output_port(self, CameraOutputPort::Video, config) }
    }
    */
}


#[repr(isize)]
enum CameraOutputPort {
    Preview = 0,
    Video = 1,
    Capture = 2
}

//------------------------------------------------------------------------------------------------------------------------------

/// Port buffer size configuration policy
#[derive(Debug, Clone, Copy)]
pub enum BufferSizePolicy {
    /// Set to recommended, or to minimum, whichever is higher
    Recommended,
    /// Set to expicit this number, or to minimum, whichever is higher
    Explicit(u32)
}

/// Port buffer count configuration policy
#[derive(Debug, Clone, Copy)]
pub enum BufferCountPolicy {
    /// Set to recommended, or to minimum, whichever is higher
    Recommended,
    /// Set to expicit this number, or to minimum, whichever is higher
    Explicit(u32)
}



/// Generic port configuration data
/// 
/// Passed to ComponentPort::configure
#[derive(Debug, Clone, Copy)]
pub struct GenericPortConfig {
    pub encoding: u32,
    pub encoding_variant: u32,
    pub es_video_width: u32,
    pub es_video_height: u32,
    pub es_video_crop_x: i32,
    pub es_video_crop_y: i32, 
    pub es_video_crop_width: i32,
    pub es_video_crop_height: i32, 
    pub es_video_frame_rate_num: i32, 
    pub es_video_frame_rate_den: i32,
    pub buffer_count_policy: BufferCountPolicy,
    pub buffer_size_policy: BufferSizePolicy,
}

impl PortConfig for GenericPortConfig {
    unsafe fn apply_format(&self, port: *mut ffi::MMAL_PORT_T) {
        let format = &mut (*(*port).format);
        // On firmware prior to June 2016, camera and video_splitter
        // had BGR24 and RGB24 support reversed.
        format.encoding = fix_encoding(port, self.encoding);
        format.encoding_variant = self.encoding_variant;

        let mut es = &mut (*format.es);
        es.video.width = self.es_video_width;
        es.video.height = self.es_video_height;
        es.video.crop.x = self.es_video_crop_x;
        es.video.crop.y = self.es_video_crop_y;
        es.video.crop.width = self.es_video_crop_width;
        es.video.crop.height = self.es_video_crop_height;
        es.video.frame_rate.num = self.es_video_frame_rate_num;
        es.video.frame_rate.den = self.es_video_frame_rate_den;
    }

    unsafe fn apply_buffer_policy(&self, port: *mut ffi::MMAL_PORT_T) {
        let port = &mut *port;
        match self.buffer_count_policy {
            BufferCountPolicy::Recommended => {
                port.buffer_num = port.buffer_num_recommended;
                if port.buffer_num < port.buffer_num_min { 
                    port.buffer_num = port.buffer_num_min;
                }
            }
            BufferCountPolicy::Explicit(n) => {
                port.buffer_num = n;
                if port.buffer_num < port.buffer_num_min { 
                    port.buffer_num = port.buffer_num_min;
                }                
            }
        }
        match self.buffer_size_policy {
            BufferSizePolicy::Recommended => {
                port.buffer_size = port.buffer_size_recommended;
                if port.buffer_size < port.buffer_size_min { 
                    port.buffer_size = port.buffer_size_min;
                }
            }
            BufferSizePolicy::Explicit(n) => {
                port.buffer_size = n;
                if port.buffer_size < port.buffer_size_min { 
                    port.buffer_size = port.buffer_size_min;
                }               
            }
        }
    }
}


/// Generic camera port configuration template
/// 
/// Should be fed to ComponentPort::configure after optional adjustments
pub const fn camera_port_config(width: u32, height: u32) -> GenericPortConfig {
    GenericPortConfig {
        encoding: ffi::MMAL_ENCODING_OPAQUE,
        encoding_variant: 0,
        es_video_width: ffi::vcos_align_up(width, 32),
        es_video_height: ffi::vcos_align_up(height, 16),
        es_video_crop_x: 0,
        es_video_crop_y: 0,
        es_video_crop_width: width as i32,
        es_video_crop_height: height as i32,
        es_video_frame_rate_num: 0,
        es_video_frame_rate_den: 1,
        buffer_count_policy: BufferCountPolicy::Explicit(3),
        buffer_size_policy: BufferSizePolicy::Recommended,
    }
}

/// full FOV 4:3 mode
pub const CAMERA_PORT_CONFIG_1024X768: GenericPortConfig = camera_port_config(1024, 768);

/// 320x240, good for streaming
pub const CAMERA_PORT_CONFIG_320X240: GenericPortConfig = camera_port_config(320, 240);


/* 
/// Default preview port configuration
pub fn camera_preview_port_config() -> GenericPortConfig {
    camera_port_config_1024x768()
}

/// Video port configuration
pub fn camera_video_port_config() -> GenericPortConfig {
    camera_port_config_1024x768()
}

/// Capture (still) port configuration
pub fn camera_capture_port_config() -> GenericPortConfig {
    camera_port_config(320, 240)
}
*/

/*
    /*
    unsafe fn post_commit(&self, port: *mut ffi::MMAL_PORT_T) -> Result<()> {
        let port = &mut *port;
        port.buffer_num = port.buffer_num_recommended;
        port.buffer_size = port.buffer_size_recommended;
        Ok(())
    }
    */
 */

//------------------------------------------------------------------------------------------------------------------------------

pub struct CameraControlPort;
impl ComponentPort for CameraControlPort {
    type E = CameraEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.control_port()
    }

    fn name() -> &'static str { "camera control port" }
}

pub struct CameraPreviewPort;
impl ComponentPort for CameraPreviewPort {
    type E = CameraEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.output_port_n(CameraOutputPort::Preview as isize)
    }

    fn name() -> &'static str { "camera preview port" }
}


pub struct CameraVideoPort;
impl ComponentPort for CameraVideoPort {
    type E = CameraEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.output_port_n(CameraOutputPort::Video as isize)
    }

    fn name() -> &'static str { "camera video port" }
}

pub struct CameraCapturePort;
impl ComponentPort for CameraCapturePort {
    type E = CameraEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.output_port_n(CameraOutputPort::Capture as isize)
    }

    fn name() -> &'static str { "camera video port" }
}


//------------------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum CameraTimestampMode {
    /// Always timestamp frames as 0
    Zero = ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_ZERO,
    /// Use the raw STC value for the frame timestamp
    RawSTC = ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_RAW_STC,
    /// Use the STC timestamp but subtract the timestamp
    ///  of the first frame sent to give a zero based timestamp
    ResetSTC = ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_RESET_STC
}

impl Default for CameraTimestampMode {
    fn default() -> Self { Self::Zero }
}

impl From<u32> for CameraTimestampMode {
    fn from(value: u32) -> Self {
        match value {
            ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_ZERO => Self::Zero,
            ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_RAW_STC => Self::RawSTC,
            ffi::MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T::MMAL_PARAM_TIMESTAMP_MODE_RESET_STC => Self::ResetSTC,
            o => panic!("Invalid u32 value for CameraTimestampMode: {o}")
        }
    }
}


/// Camera configuration settings
#[derive(Clone, Debug, Default)]
pub struct CameraConfig {
    /// Max size of stills capture - X
    pub max_stills_w: u32,
    /// Max size of stills capture - Y
    pub max_stills_h: u32,
    /// Allow YUV422 stills capture
    pub stills_yuv422: bool,
    /// Continuous or one shot stills captures
    pub one_shot_stills: bool,
    /// Max size of the preview or video capture frames - X
    pub max_preview_video_w: u32,
    /// Max size of the preview or video capture frames - Y
    pub max_preview_video_h: u32,
    pub num_preview_video_frames: u32,
    /// Sets the height of the circular buffer for stills capture
    pub stills_capture_circular_buffer_height: u32,
    /// Allows preview/encode to resume as fast as possible after the stills input frame
    /// has been received, and then processes the still frame in the background
    /// whilst preview/encode has resumed.
    /// Actual mode is controlled by MMAL_PARAMETER_CAPTURE_MODE
    pub fast_preview_resume: bool,
    pub use_stc_timestamp: CameraTimestampMode,
}

impl CameraConfig {
    pub fn from_instance_info(instance: &CameraInstanceInfo) -> Self {
        Self { 
            max_stills_w: instance.max_width, 
            max_stills_h: instance.max_height, 
            stills_yuv422: false, 
            one_shot_stills: true, 
            max_preview_video_w: instance.max_width, 
            max_preview_video_h: instance.max_height, 
            num_preview_video_frames: 1, 
            stills_capture_circular_buffer_height: 0, 
            fast_preview_resume: false, 
            use_stc_timestamp: CameraTimestampMode::ResetSTC 
        }
    }
}

pub struct CameraConfigInnerType {
    inner: ffi::MMAL_PARAMETER_CAMERA_CONFIG_T
}

impl Default for CameraConfigInnerType {
    fn default() -> Self { unsafe {
        let mut cfg: ffi::MMAL_PARAMETER_CAMERA_CONFIG_T = mem::zeroed();
        cfg.hdr.id = ffi::MMAL_PARAMETER_CAMERA_CONFIG as u32;
        cfg.hdr.size = mem::size_of::<ffi::MMAL_PARAMETER_CAMERA_CONFIG_T>() as u32;
        Self { inner: cfg }
    } }
}

impl InnerParamType for CameraConfigInnerType {
    fn name() -> &'static str {"CameraConfig" }

    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_get(port, &mut self.inner.hdr)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set(port, &self.inner.hdr)
    }
}

impl UpdateFrom<CameraConfig> for CameraConfigInnerType {
    fn update_from(&mut self, source: CameraConfig) {
        fn cbool(w: bool) -> u32 { if w { ffi::MMAL_TRUE } else { ffi::MMAL_FALSE } }
        let mut cfg = &mut self.inner;
        cfg.max_stills_w = source.max_stills_w;
        cfg.max_stills_h = source.max_stills_h;
        cfg.stills_yuv422 = cbool(source.stills_yuv422);
        cfg.one_shot_stills = cbool(source.one_shot_stills);
        cfg.max_preview_video_w = source.max_preview_video_w;
        cfg.max_preview_video_h = source.max_preview_video_h;
        cfg.num_preview_video_frames = source.num_preview_video_frames;
        cfg.stills_capture_circular_buffer_height = source.stills_capture_circular_buffer_height;
        cfg.fast_preview_resume = cbool(source.fast_preview_resume);
        cfg.use_stc_timestamp = source.use_stc_timestamp as u32;
    }
}

impl From<&CameraConfigInnerType> for CameraConfig {
    fn from(value: &CameraConfigInnerType) -> Self {
        fn cbool(w: u32) -> bool { w != ffi::MMAL_FALSE }
        Self {
            max_stills_w: value.inner.max_stills_w,
            max_stills_h: value.inner.max_stills_h,
            stills_yuv422: cbool(value.inner.stills_yuv422),
            one_shot_stills: cbool(value.inner.one_shot_stills),
            max_preview_video_w: value.inner.max_preview_video_w,
            max_preview_video_h: value.inner.max_preview_video_h,
            num_preview_video_frames: value.inner.num_preview_video_frames,
            stills_capture_circular_buffer_height: value.inner.stills_capture_circular_buffer_height,
            fast_preview_resume: cbool(value.inner.fast_preview_resume),
            use_stc_timestamp: value.inner.use_stc_timestamp.into(),
        }
        
    }
}

idp!{MMAL_PARAMETER_CAMERA_CONFIG}
/// Basic camera configuration
pub type PCameraConfig = Param<CameraControlPort, CameraConfigInnerType>;

//------------------------------------------------------------------------------------------------------------------------------

idp!{MMAL_PARAMETER_SATURATION}
/// Saturation [-100, 100]
pub type PSaturation = Param<CameraControlPort, Rational<MMAL_PARAMETER_SATURATION>>;

idp!{MMAL_PARAMETER_SHARPNESS}
/// Sharpness [-100, 100]
pub type PSharpness = Param<CameraControlPort, Rational<MMAL_PARAMETER_SHARPNESS>>;

idp!{MMAL_PARAMETER_CONTRAST}
/// Contrast [-100, 100]
pub type PContrast = Param<CameraControlPort, Rational<MMAL_PARAMETER_CONTRAST>>;

idp!{MMAL_PARAMETER_BRIGHTNESS}
/// Brightness [0, 100]
pub type PBrightness = Param<CameraControlPort, Rational<MMAL_PARAMETER_BRIGHTNESS>>;

idp!{MMAL_PARAMETER_ISO}
/// ISO
pub type PIso = Param<CameraControlPort, Uint32<MMAL_PARAMETER_ISO>>;

idp!{MMAL_PARAMETER_SHUTTER_SPEED}
/// Shutter speed in mcroseconds
pub type PShutterSpeed = Param<CameraControlPort, Uint32<MMAL_PARAMETER_SHUTTER_SPEED>>;

idp!{MMAL_PARAMETER_CAMERA_NUM}
/// Camera ordinal
pub type PCameraNum = Param<CameraControlPort, Int32<MMAL_PARAMETER_CAMERA_NUM>>;

idp!{MMAL_PARAMETER_CAPTURE}
/// Activate/deactivate capture
pub type PCapture = Param<CameraCapturePort, Boolean<MMAL_PARAMETER_CAPTURE>>;
pub type PCaptureVideo = Param<CameraVideoPort, Boolean<MMAL_PARAMETER_CAPTURE>>;

/*
   result += raspicamcontrol_set_exposure_mode(camera, params->exposureMode);
int raspicamcontrol_set_exposure_mode(MMAL_COMPONENT_T *camera, MMAL_PARAM_EXPOSUREMODE_T mode)
Set exposure mode for images

Parameters:
camera – Pointer to camera component
mode – Exposure mode to set from 
    - MMAL_PARAM_EXPOSUREMODE_OFF, - MMAL_PARAM_EXPOSUREMODE_AUTO, - MMAL_PARAM_EXPOSUREMODE_NIGHT, 
    - MMAL_PARAM_EXPOSUREMODE_NIGHTPREVIEW, - MMAL_PARAM_EXPOSUREMODE_BACKLIGHT, - MMAL_PARAM_EXPOSUREMODE_SPOTLIGHT, 
    - MMAL_PARAM_EXPOSUREMODE_SPORTS, - MMAL_PARAM_EXPOSUREMODE_SNOW, - MMAL_PARAM_EXPOSUREMODE_BEACH, 
    - MMAL_PARAM_EXPOSUREMODE_VERYLONG, - MMAL_PARAM_EXPOSUREMODE_FIXEDFPS, - MMAL_PARAM_EXPOSUREMODE_ANTISHAKE, 
    - MMAL_PARAM_EXPOSUREMODE_FIREWORKS,
 */

enumize!{ExposureMode,
    Off => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_OFF,
    Auto => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_AUTO,
    Night => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_NIGHT,
    NightPreview => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_NIGHTPREVIEW,
    Backlght => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_BACKLIGHT,
    Spotlight => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_SPOTLIGHT,
    Sports => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_SPORTS,
    Snow => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_SNOW,
    Beach => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_BEACH,
    VeryLong => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_VERYLONG,
    FixedFPS => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_FIXEDFPS,
    AntiShake => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_ANTISHAKE,
    Fireworks => MMAL_PARAM_EXPOSUREMODE_T_MMAL_PARAM_EXPOSUREMODE_FIREWORKS
}

enumerated_inner_type!{ExposureModeInnerType, ExposureMode, MMAL_PARAMETER_EXPOSUREMODE_T, MMAL_PARAMETER_EXPOSURE_MODE}
idp!{MMAL_PARAMETER_EXPOSURE_MODE}
/// Exposure mode
pub type PExposureMode = Param<CameraCapturePort, ExposureModeInnerType>;


/*
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_metering_mode(camera, params->exposureMeterMode);
int raspicamcontrol_set_metering_mode(MMAL_COMPONENT_T *camera, MMAL_PARAM_EXPOSUREMETERINGMODE_T m_mode)
Adjust the metering mode for images

Parameters:
camera – Pointer to camera component
saturation – Value from following 
    - MMAL_PARAM_EXPOSUREMETERINGMODE_AVERAGE, - MMAL_PARAM_EXPOSUREMETERINGMODE_SPOT, - MMAL_PARAM_EXPOSUREMETERINGMODE_BACKLIT, 
    - MMAL_PARAM_EXPOSUREMETERINGMODE_MATRIX
*/

enumize!{ExposureMeteringMode,
    Average => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_AVERAGE,
    Spot => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_SPOT,
    Backlit => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_BACKLIT, 
    Matrix => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_MATRIX
}

enumerated_inner_type!{ExposureMeteringModeInnerType, ExposureMeteringMode, MMAL_PARAMETER_EXPOSUREMETERINGMODE_T, 
    MMAL_PARAMETER_EXP_METERING_MODE}

idp!{MMAL_PARAMETER_EXP_METERING_MODE}
/// Exposure mode
pub type PExposureMeteringMode = Param<CameraCapturePort, ExposureMeteringModeInnerType>;


// TODO implement all the following parameters

            /*
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_exposure_compensation(camera, params->exposureCompensation);
int raspicamcontrol_set_exposure_compensation(MMAL_COMPONENT_T *camera, int exp_comp)
Adjust the exposure compensation for images (EV)

Parameters:
camera – Pointer to camera component
exp_comp – Value to adjust, -10 to +10

Returns:
0 if successful, non-zero if any parameters out of range


---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_awb_mode(camera, params->awbMode);
int raspicamcontrol_set_awb_mode(MMAL_COMPONENT_T *camera, MMAL_PARAM_AWBMODE_T awb_mode)
Set the aWB (auto white balance) mode for images

Parameters:
camera – Pointer to camera component
awb_mode – Value to set from 
    - MMAL_PARAM_AWBMODE_OFF, - MMAL_PARAM_AWBMODE_AUTO, - MMAL_PARAM_AWBMODE_SUNLIGHT, 
    - MMAL_PARAM_AWBMODE_CLOUDY, - MMAL_PARAM_AWBMODE_SHADE, - MMAL_PARAM_AWBMODE_TUNGSTEN, 
    - MMAL_PARAM_AWBMODE_FLUORESCENT, - MMAL_PARAM_AWBMODE_INCANDESCENT, - MMAL_PARAM_AWBMODE_FLASH, 
    - MMAL_PARAM_AWBMODE_HORIZON,

Returns:
0 if successful, non-zero if any parameters out of range

---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_awb_gains(camera, params->awb_gains_r, params->awb_gains_b);
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_imageFX(camera, params->imageEffect);
int raspicamcontrol_set_imageFX(MMAL_COMPONENT_T *camera, MMAL_PARAM_IMAGEFX_T imageFX)
Set the image effect for the images

Parameters:
camera – Pointer to camera component
imageFX – Value from - MMAL_PARAM_IMAGEFX_NONE, - MMAL_PARAM_IMAGEFX_NEGATIVE, - MMAL_PARAM_IMAGEFX_SOLARIZE, - MMAL_PARAM_IMAGEFX_POSTERIZE, - MMAL_PARAM_IMAGEFX_WHITEBOARD, - MMAL_PARAM_IMAGEFX_BLACKBOARD, - MMAL_PARAM_IMAGEFX_SKETCH, - MMAL_PARAM_IMAGEFX_DENOISE, - MMAL_PARAM_IMAGEFX_EMBOSS, - MMAL_PARAM_IMAGEFX_OILPAINT, - MMAL_PARAM_IMAGEFX_HATCH, - MMAL_PARAM_IMAGEFX_GPEN, - MMAL_PARAM_IMAGEFX_PASTEL, - MMAL_PARAM_IMAGEFX_WATERCOLOUR, - MMAL_PARAM_IMAGEFX_FILM, - MMAL_PARAM_IMAGEFX_BLUR, - MMAL_PARAM_IMAGEFX_SATURATION, - MMAL_PARAM_IMAGEFX_COLOURSWAP, - MMAL_PARAM_IMAGEFX_WASHEDOUT, - MMAL_PARAM_IMAGEFX_POSTERISE, - MMAL_PARAM_IMAGEFX_COLOURPOINT, - MMAL_PARAM_IMAGEFX_COLOURBALANCE, - MMAL_PARAM_IMAGEFX_CARTOON,

Returns:
0 if successful, non-zero if any parameters out of range
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_colourFX(camera, &params->colourEffects);
int raspicamcontrol_set_colourFX(MMAL_COMPONENT_T *camera, const MMAL_PARAM_COLOURFX_T *colourFX)
Set the colour effect for images (Set UV component)

Parameters:
camera – Pointer to camera component
colourFX – Contains enable state and U and V numbers to set (e.g. 128,128 = Black and white)

Returns:
0 if successful, non-zero if any parameters out of range
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_rotation(camera, params->rotation);
int raspicamcontrol_set_rotation(MMAL_COMPONENT_T *camera, int rotation)
Set the rotation of the image

Parameters:
camera – Pointer to camera component
rotation – Degree of rotation (any number, but will be converted to 0,90,180 or 270 only)

Returns:
0 if successful, non-zero if any parameters out of range
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_flips(camera, params->hflip, params->vflip);
int raspicamcontrol_set_flips(MMAL_COMPONENT_T *camera, int hflip, int vflip)
Set the flips state of the image

Parameters:
camera – Pointer to camera component
hflip – If true, horizontally flip the image
vflip – If true, vertically flip the image

Returns:
0 if successful, non-zero if any parameters out of range
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_ROI(camera, params->roi);
int raspicamcontrol_set_ROI(MMAL_COMPONENT_T *camera, PARAM_FLOAT_RECT_T rect)
Set the ROI of the sensor to use for captures/preview

Parameters:
camera – Pointer to camera component
rect – Normalised coordinates of ROI rectangle

Returns:
0 if successful, non-zero if any parameters out of range
---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_shutter_speed(camera, params->shutter_speed);
int raspicamcontrol_set_shutter_speed(MMAL_COMPONENT_T *camera, int speed)
Adjust the exposure time used for images

Parameters:
camera – Pointer to camera component
shutter – speed in microseconds

Returns:
0 if successful, non-zero if any parameters out of range


---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_DRC(camera, params->drc_level);
int raspicamcontrol_set_DRC(MMAL_COMPONENT_T *camera, MMAL_PARAMETER_DRC_STRENGTH_T strength)
Adjust the Dynamic range compression level

Parameters:
camera – Pointer to camera component
strength – Strength of DRC to apply MMAL_PARAMETER_DRC_STRENGTH_OFF MMAL_PARAMETER_DRC_STRENGTH_LOW MMAL_PARAMETER_DRC_STRENGTH_MEDIUM MMAL_PARAMETER_DRC_STRENGTH_HIGH

Returns:
0 if successful, non-zero if any parameters out of range

---------------------------------------------------------------------------------------------------------------
   result += raspicamcontrol_set_stats_pass(camera, params->stats_pass);
             */

