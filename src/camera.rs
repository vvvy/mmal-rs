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

    fn name() -> &'static str { "camera capture port" }
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

pub fn bool_rust_to_mmal_u32(w: bool) -> u32 { if w { ffi::MMAL_TRUE } else { ffi::MMAL_FALSE } }
pub fn bool_mmal_u32_to_rust(w: u32) -> bool { w != ffi::MMAL_FALSE }

pub fn bool_rust_to_mmal(w: bool) -> i32 { (if w { ffi::MMAL_TRUE } else { ffi::MMAL_FALSE }) as i32 }
pub fn bool_mmal_to_rust(w: i32) -> bool { w != (ffi::MMAL_FALSE as i32) }

impl Apply<CameraConfig> for CameraConfigInnerType {
    fn apply(&mut self, source: CameraConfig) {
        let cbool = bool_rust_to_mmal_u32;
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
        let cbool = bool_mmal_u32_to_rust;
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
/// Activate/deactivate capture
pub type PCaptureVideo = Param<CameraVideoPort, Boolean<MMAL_PARAMETER_CAPTURE>>;

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
/// Set exposure mode for images
/// 
/// Native type: `MMAL_PARAM_EXPOSUREMODE_T`
/// 
/// mode: Exposure mode to set from 
/// * MMAL_PARAM_EXPOSUREMODE_OFF
/// * MMAL_PARAM_EXPOSUREMODE_AUTO
/// * MMAL_PARAM_EXPOSUREMODE_NIGHT
/// * MMAL_PARAM_EXPOSUREMODE_NIGHTPREVIEW
/// * MMAL_PARAM_EXPOSUREMODE_BACKLIGHT
/// * MMAL_PARAM_EXPOSUREMODE_SPOTLIGHT
/// * MMAL_PARAM_EXPOSUREMODE_SPORTS
/// * MMAL_PARAM_EXPOSUREMODE_SNOW
/// * MMAL_PARAM_EXPOSUREMODE_BEACH
/// * MMAL_PARAM_EXPOSUREMODE_VERYLONG
/// * MMAL_PARAM_EXPOSUREMODE_FIXEDFPS
/// * MMAL_PARAM_EXPOSUREMODE_ANTISHAKE
/// * MMAL_PARAM_EXPOSUREMODE_FIREWORKS
pub type PExposureMode = Param<CameraControlPort, ExposureModeInnerType>;

enumize!{ExposureMeteringMode,
    Average => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_AVERAGE,
    Spot => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_SPOT,
    Backlit => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_BACKLIT, 
    Matrix => MMAL_PARAM_EXPOSUREMETERINGMODE_T_MMAL_PARAM_EXPOSUREMETERINGMODE_MATRIX
}
enumerated_inner_type!{ExposureMeteringModeInnerType, ExposureMeteringMode, MMAL_PARAMETER_EXPOSUREMETERINGMODE_T, 
    MMAL_PARAMETER_EXP_METERING_MODE}
idp!{MMAL_PARAMETER_EXP_METERING_MODE}
/// Adjust the metering mode for images
/// 
/// Native type: `MMAL_PARAM_EXPOSUREMETERINGMODE_T`
/// 
/// saturation – Value from following:
/// 
/// * MMAL_PARAM_EXPOSUREMETERINGMODE_AVERAGE
/// * MMAL_PARAM_EXPOSUREMETERINGMODE_SPOT
/// * MMAL_PARAM_EXPOSUREMETERINGMODE_BACKLIT
/// * MMAL_PARAM_EXPOSUREMETERINGMODE_MATRIX
pub type PExposureMeteringMode = Param<CameraControlPort, ExposureMeteringModeInnerType>;

idp!{MMAL_PARAMETER_EXPOSURE_COMP}
/// Adjust the exposure compensation for images (EV)
/// exp_comp – Value to adjust, -10 to +10
pub type PExposureCompensation = Param<CameraControlPort, Uint32<MMAL_PARAMETER_EXPOSURE_COMP>>;


enumize!{AwbMode,
    Off => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_OFF, 
    Auto => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_AUTO, 
    Sunlight => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_SUNLIGHT, 
    Cloudy => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_CLOUDY, 
    Shade => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_SHADE, 
    Tungsten => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_TUNGSTEN, 
    Fluorescent => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_FLUORESCENT, 
    Incandescent => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_INCANDESCENT, 
    Flash => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_FLASH, 
    Horizon => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_HORIZON,
    GreyWorld => MMAL_PARAM_AWBMODE_T_MMAL_PARAM_AWBMODE_GREYWORLD
}
enumerated_inner_type!{AwbModeInnerType, AwbMode, MMAL_PARAMETER_AWBMODE_T, MMAL_PARAMETER_AWB_MODE}
idp!{MMAL_PARAMETER_AWB_MODE}
/// Set the aWB (auto white balance) mode for images
/// 
/// Native type: MMAL_PARAMETER_AWBMODE_T
/// 
/// awb_mode – Value to set from
/// * MMAL_PARAM_AWBMODE_OFF
/// * MMAL_PARAM_AWBMODE_AUTO
/// * MMAL_PARAM_AWBMODE_SUNLIGHT
/// * MMAL_PARAM_AWBMODE_CLOUDY
/// * MMAL_PARAM_AWBMODE_SHADE
/// * MMAL_PARAM_AWBMODE_TUNGSTEN
/// * MMAL_PARAM_AWBMODE_FLUORESCENT
/// * MMAL_PARAM_AWBMODE_INCANDESCENT
/// * MMAL_PARAM_AWBMODE_FLASH
/// * MMAL_PARAM_AWBMODE_HORIZON
pub type PAwbMode = Param<CameraControlPort, AwbModeInnerType>;


pub struct AwbGainsInnerType {
    inner: ffi::MMAL_PARAMETER_AWB_GAINS_T,
}

impl_inner_param_default!{AwbGainsInnerType, MMAL_PARAMETER_AWB_GAINS_T, MMAL_PARAMETER_CUSTOM_AWB_GAINS}
impl_inner_param_type!{AwbGainsInnerType}

impl AwbGainsInnerType {
    pub fn set(&mut self, r_gain: f64, b_gain: f64) {
        const C: i32 = 0x1_0000;
        self.inner.r_gain.num = (r_gain * C as f64) as i32;
        self.inner.r_gain.den = C;

        self.inner.b_gain.num = (b_gain * C as f64) as i32;
        self.inner.b_gain.den = C;
    }

    pub fn new(r_gain: f64, b_gain: f64) -> Self {
        let mut rv = Self::default();
        rv.set(r_gain, b_gain);
        rv
    }
}

impl Apply<(f64, f64)> for AwbGainsInnerType {
    fn apply(&mut self, (r_gain, b_gain): (f64, f64)) {
        self.set(r_gain, b_gain)
    }
}

impl Into<(f64, f64)> for &'_ AwbGainsInnerType {
    fn into(self) -> (f64, f64) {
        (self.inner.r_gain.num as f64/self.inner.r_gain.den as f64,
        self.inner.b_gain.num as f64/self.inner.b_gain.den as f64
        )
    }
}

/// aWB gains - (r, b)
pub type PAwbGains = Param<CameraControlPort, AwbGainsInnerType>;


enumize!{ImageFX,
    None => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_NONE,
    Negative => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_NEGATIVE,
    Solarize => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_SOLARIZE,
    Posterize => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_POSTERIZE,
    Whiteboard => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_WHITEBOARD,
    BlackBoard => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_BLACKBOARD,
    Sketch => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_SKETCH,
    Denoise => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_DENOISE, 
    Emboss => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_EMBOSS, 
    OilPaint => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_OILPAINT, 
    Hatch => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_HATCH,
    Gpen => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_GPEN,
    Pastel => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_PASTEL,
    WaterColour => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_WATERCOLOUR,
    Film => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_FILM,
    Blur => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_BLUR,
    Saturation => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_SATURATION,
    ColourSwap => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_COLOURSWAP,
    WashedOut => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_WASHEDOUT,
    Posterise => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_POSTERISE,
    ColourPoint => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_COLOURPOINT,
    ColourBalance => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_COLOURBALANCE,
    Cartoon => MMAL_PARAM_IMAGEFX_T_MMAL_PARAM_IMAGEFX_CARTOON
}
enumerated_inner_type!{ImageFXInnerType, ImageFX, MMAL_PARAMETER_IMAGEFX_T, MMAL_PARAMETER_IMAGE_EFFECT}
idp!{MMAL_PARAMETER_IMAGE_EFFECT}
/// Set the image effect for the images
/// 
/// imageFX – Value from
/// * MMAL_PARAM_IMAGEFX_NONE
/// * MMAL_PARAM_IMAGEFX_NEGATIVE
/// * MMAL_PARAM_IMAGEFX_SOLARIZE
/// * MMAL_PARAM_IMAGEFX_POSTERIZE
/// * MMAL_PARAM_IMAGEFX_WHITEBOARD
/// * MMAL_PARAM_IMAGEFX_BLACKBOARD
/// * MMAL_PARAM_IMAGEFX_SKETCH
/// * MMAL_PARAM_IMAGEFX_DENOISE
/// * MMAL_PARAM_IMAGEFX_EMBOSS
/// * MMAL_PARAM_IMAGEFX_OILPAINT
/// * MMAL_PARAM_IMAGEFX_HATCH
/// * MMAL_PARAM_IMAGEFX_GPEN
/// * MMAL_PARAM_IMAGEFX_PASTEL
/// * MMAL_PARAM_IMAGEFX_WATERCOLOUR
/// * MMAL_PARAM_IMAGEFX_FILM
/// * MMAL_PARAM_IMAGEFX_BLUR
/// * MMAL_PARAM_IMAGEFX_SATURATION
/// * MMAL_PARAM_IMAGEFX_COLOURSWAP
/// * MMAL_PARAM_IMAGEFX_WASHEDOUT
/// * MMAL_PARAM_IMAGEFX_POSTERISE
/// * MMAL_PARAM_IMAGEFX_COLOURPOINT
/// * MMAL_PARAM_IMAGEFX_COLOURBALANCE
/// * MMAL_PARAM_IMAGEFX_CARTOON
pub type PImageFX = Param<CameraControlPort, ImageFXInnerType>;


pub struct ColourFxInnerType {
    inner: ffi::MMAL_PARAMETER_COLOURFX_T,
}

impl_inner_param_default!{ColourFxInnerType, MMAL_PARAMETER_COLOURFX_T, MMAL_PARAMETER_COLOUR_EFFECT}
impl_inner_param_type!{ColourFxInnerType}

impl ColourFxInnerType {
    pub fn set(&mut self, enable: bool, u: u32, v: u32) {
        self.inner.enable = bool_rust_to_mmal(enable);
        self.inner.u = u; 
        self.inner.v = v;
    }

    pub fn new(enable: bool, u: u32, v: u32) -> Self {
        let mut rv = Self::default();
        rv.set(enable, u, v);
        rv
    }
}

impl Apply<(bool, u32, u32)> for ColourFxInnerType {
    fn apply(&mut self, (enable, u, v): (bool, u32, u32)) {
        self.set(enable, u, v);
    }
}

impl Into<(bool, u32, u32)> for &'_ ColourFxInnerType {
    fn into(self) -> (bool, u32, u32) {
        (bool_mmal_to_rust(self.inner.enable), self.inner.u, self.inner.v)
    }
}

/// Set the colour effect for images (Set UV component)
/// 
/// colourFX – Contains enable state and U and V numbers to set (e.g. 128,128 = Black and white)
pub type PColourFX = Param<CameraControlPort, ColourFxInnerType>;


#[repr(i32)]
pub enum Rotation {
    R0 = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270
}
idp!{MMAL_PARAMETER_ROTATION}
/// Degree of rotation, must be one of: 0,90,180 or 270
pub type PRotationVideo = Param<CameraVideoPort, Int32<MMAL_PARAMETER_ROTATION>>;
/// Degree of rotation, must be one of: 0,90,180 or 270
pub type PRotationPreview = Param<CameraPreviewPort, Int32<MMAL_PARAMETER_ROTATION>>;
/// Degree of rotation, must be one of: 0,90,180 or 270
pub type PRotationCapture = Param<CameraCapturePort, Int32<MMAL_PARAMETER_ROTATION>>;


enumize!{Mirror,
    None => MMAL_PARAM_MIRROR_T_MMAL_PARAM_MIRROR_NONE,
    Both => MMAL_PARAM_MIRROR_T_MMAL_PARAM_MIRROR_BOTH,
    Horizontal => MMAL_PARAM_MIRROR_T_MMAL_PARAM_MIRROR_HORIZONTAL,
    Vertical => MMAL_PARAM_MIRROR_T_MMAL_PARAM_MIRROR_VERTICAL
}
enumerated_inner_type!{MirrorInnerType, Mirror, MMAL_PARAMETER_MIRROR_T, MMAL_PARAMETER_MIRROR}
idp!{MMAL_PARAMETER_MIRROR}
/// Set the mirroring state of the image
pub type PMirrorVideo = Param<CameraVideoPort, MirrorInnerType>;
/// Set the mirroring state of the image
pub type PMirrorPreview = Param<CameraPreviewPort, MirrorInnerType>;
/// Set the mirroring state of the image
pub type PMirrorCapture = Param<CameraCapturePort, MirrorInnerType>;


pub struct CropInnerType {
    inner: ffi::MMAL_PARAMETER_INPUT_CROP_T,
}

impl_inner_param_default!{CropInnerType, MMAL_PARAMETER_INPUT_CROP_T, MMAL_PARAMETER_INPUT_CROP}
impl_inner_param_type!{CropInnerType}

impl CropInnerType {
    pub fn set(&mut self, (x, width): (i32, i32), (y, height): (i32, i32)) {
        self.inner.rect.x = x;
        self.inner.rect.width = width;
        self.inner.rect.y = y;
        self.inner.rect.height = height;
    }

    pub fn new(xw: (i32, i32), yh: (i32, i32)) -> Self {
        let mut rv = Self::default();
        rv.set(xw, yh);
        rv
    }
}

impl Apply<((i32, i32), (i32, i32))> for CropInnerType {
    fn apply(&mut self, (xw, yh): ((i32, i32), (i32, i32))) {
        self.set(xw, yh);
    }
}

impl Into<((i32, i32), (i32, i32))> for &'_ CropInnerType {
    fn into(self) -> ((i32, i32), (i32, i32)) {
        (
            (self.inner.rect.x, self.inner.rect.width),
            (self.inner.rect.y, self.inner.rect.height)
        )
    }
}

///Set the ROI of the sensor to use for captures/preview
pub type PROI = Param<CameraControlPort, MirrorInnerType>;

enumize!{DRC,
    Off => MMAL_PARAMETER_DRC_STRENGTH_T_MMAL_PARAMETER_DRC_STRENGTH_OFF,
    Low => MMAL_PARAMETER_DRC_STRENGTH_T_MMAL_PARAMETER_DRC_STRENGTH_LOW,
    Medium => MMAL_PARAMETER_DRC_STRENGTH_T_MMAL_PARAMETER_DRC_STRENGTH_MEDIUM,
    High => MMAL_PARAMETER_DRC_STRENGTH_T_MMAL_PARAMETER_DRC_STRENGTH_HIGH
}
enumerated_inner_type!{DRCInnerType, DRC, MMAL_PARAMETER_DRC_T, MMAL_PARAMETER_DYNAMIC_RANGE_COMPRESSION, strength}
idp!{MMAL_PARAMETER_DYNAMIC_RANGE_COMPRESSION}
/// Adjust the Dynamic range compression level
pub type PDRC = Param<CameraControlPort, DRCInnerType>;

idp!{MMAL_PARAMETER_CAPTURE_STATS_PASS}
/// Stats pass
pub type PStatsPass = Param<CameraControlPort, Boolean<MMAL_PARAMETER_CAPTURE_STATS_PASS>>;


pub struct StereoModeInnerType {
    inner: ffi::MMAL_PARAMETER_STEREOSCOPIC_MODE_T,
}

enumize!{StereoMode, 
    None => MMAL_STEREOSCOPIC_MODE_T_MMAL_STEREOSCOPIC_MODE_NONE,
    SideBySide => MMAL_STEREOSCOPIC_MODE_T_MMAL_STEREOSCOPIC_MODE_SIDE_BY_SIDE,
    TopBottom => MMAL_STEREOSCOPIC_MODE_T_MMAL_STEREOSCOPIC_MODE_TOP_BOTTOM,
    Max => MMAL_STEREOSCOPIC_MODE_T_MMAL_STEREOSCOPIC_MODE_MAX
}

impl_inner_param_default!{StereoModeInnerType, MMAL_PARAMETER_STEREOSCOPIC_MODE_T, MMAL_PARAMETER_STEREOSCOPIC_MODE}
impl_inner_param_type!{StereoModeInnerType}

impl StereoModeInnerType {
    pub fn set(&mut self, mode: StereoMode, decimate: bool, swap_eyes: bool) {
        self.inner.mode = mode as u32;
        self.inner.decimate = bool_rust_to_mmal(decimate); 
        self.inner.swap_eyes = bool_rust_to_mmal(swap_eyes);
    }

    pub fn new(mode: StereoMode, decimate: bool, swap_eyes: bool) -> Self {
        let mut rv = Self::default();
        rv.set(mode, decimate, swap_eyes);
        rv
    }
}

impl Apply<(StereoMode, bool, bool)> for StereoModeInnerType {
    fn apply(&mut self, (mode, decimate, swap_eyes): (StereoMode, bool, bool)) {
        self.set(mode, decimate, swap_eyes);
    }
}

impl TryInto<(StereoMode, bool, bool)> for &'_ StereoModeInnerType {
    type Error = MmalError;
    fn try_into(self) -> Result<(StereoMode, bool, bool)> {
        Ok((
            self.inner.mode.try_into()?,
            bool_mmal_to_rust(self.inner.decimate), 
            bool_mmal_to_rust(self.inner.swap_eyes)
        ))
    }
}


/// stereo mode
pub type PStereoModePreview = Param<CameraPreviewPort, StereoModeInnerType>;
/// stereo mode
pub type PStereoModeVideo = Param<CameraVideoPort, StereoModeInnerType>;
/// stereo mode
pub type PStereoModeCapture = Param<CameraCapturePort, StereoModeInnerType>;


idp!{MMAL_PARAMETER_VIDEO_STABILISATION}
/// Set the video stabilisation flag. Only used in video mode
pub type PVideoStabilization = Param<CameraControlPort, Boolean<MMAL_PARAMETER_VIDEO_STABILISATION>>;

enumize!{FlickerAvoid, 
    Off => MMAL_PARAM_FLICKERAVOID_T_MMAL_PARAM_FLICKERAVOID_OFF,
    Auto => MMAL_PARAM_FLICKERAVOID_T_MMAL_PARAM_FLICKERAVOID_AUTO,
    Hz50 => MMAL_PARAM_FLICKERAVOID_T_MMAL_PARAM_FLICKERAVOID_50HZ,
    Hz60 => MMAL_PARAM_FLICKERAVOID_T_MMAL_PARAM_FLICKERAVOID_60HZ
}
enumerated_inner_type!{FlickerAvoidInnerType, FlickerAvoid, MMAL_PARAMETER_FLICKERAVOID_T, MMAL_PARAMETER_FLICKER_AVOID}
idp!{MMAL_PARAMETER_FLICKER_AVOID}
/// Set flicker avoid mode for images
/// * MMAL_PARAM_FLICKERAVOID_OFF
/// * MMAL_PARAM_FLICKERAVOID_AUTO
/// * MMAL_PARAM_FLICKERAVOID_50HZ
/// * MMAL_PARAM_FLICKERAVOID_60HZ
pub type PFlickerAvoid = Param<CameraControlPort, FlickerAvoidInnerType>;

idp!{MMAL_PARAMETER_ANALOG_GAIN}
/// Set analog gain (?)
pub type PAnalogGain = Param<CameraControlPort, Rational<MMAL_PARAMETER_ANALOG_GAIN>>;

idp!{MMAL_PARAMETER_DIGITAL_GAIN}
/// Set digital gain (?)
pub type PDigitalGain = Param<CameraControlPort, Rational<MMAL_PARAMETER_DIGITAL_GAIN>>;

idp!{MMAL_PARAMETER_DRAW_BOX_FACES_AND_FOCUS}
/// Set focus window on/off gain (?)
pub type PFocusWindow = Param<CameraControlPort, Boolean<MMAL_PARAMETER_DRAW_BOX_FACES_AND_FOCUS>>;


pub struct Annotate {
    pub enable: bool,
    pub show_shutter: bool,
    pub show_analog_gain: bool,
    pub show_lens: bool,
    pub show_caf: bool,
    pub show_motion: bool,
    pub show_frame_num: bool,
    pub enable_text_background: bool,
    pub custom_background_colour: bool,
    pub custom_background_y: u8,
    pub custom_background_u: u8,
    pub custom_background_v: u8,
    pub custom_text_colour: bool,
    pub custom_text_y: u8,
    pub custom_text_u: u8,
    pub custom_text_v: u8,
    pub text_size: u8,
    pub text: String,
    /// 0=centre, 1=left, 2=right
    pub justify: u32,
    /// Offset from the justification edge, x
    pub x_offset: u32,
    /// Offset from the justification edge, y
    pub y_offset: u32,
}


impl Default for Annotate {
    fn default() -> Self {
        Self { 
            enable: Default::default(), show_shutter: Default::default(), show_analog_gain: Default::default(), 
            show_lens: Default::default(), show_caf: Default::default(), show_motion: Default::default(), 
            show_frame_num: Default::default(), enable_text_background: Default::default(), 
            custom_background_colour: Default::default(), custom_background_y: Default::default(), 
            custom_background_u: Default::default(), custom_background_v: Default::default(), 
            custom_text_colour: Default::default(), custom_text_y: Default::default(), custom_text_u: Default::default(), 
            custom_text_v: Default::default(), text_size: Default::default(), text: Default::default(), 
            justify: Default::default(), x_offset: Default::default(), y_offset: Default::default() }
    }
}


pub struct AnnotateInnerType {
    inner: ffi::MMAL_PARAMETER_CAMERA_ANNOTATE_V4_T,
}

impl_inner_param_default!{AnnotateInnerType, MMAL_PARAMETER_CAMERA_ANNOTATE_V4_T, MMAL_PARAMETER_ANNOTATE}
impl_inner_param_type!{AnnotateInnerType}


impl AnnotateInnerType {
    pub fn new() -> Self { Self::default() }
}

impl Apply<&Annotate> for AnnotateInnerType {
    fn apply(&mut self, w: &Annotate) {
        use std::ffi::CString;

        self.inner.enable = bool_rust_to_mmal(w.enable);
        self.inner.show_shutter = bool_rust_to_mmal(w.show_shutter);
        self.inner.show_analog_gain = bool_rust_to_mmal(w.show_analog_gain);
        self.inner.show_lens = bool_rust_to_mmal(w.show_lens);
        self.inner.show_caf = bool_rust_to_mmal(w.show_caf);
        self.inner.show_motion = bool_rust_to_mmal(w.show_motion);
        self.inner.show_frame_num = bool_rust_to_mmal(w.show_frame_num);
        self.inner.enable_text_background = bool_rust_to_mmal(w.enable_text_background);
        self.inner.custom_background_colour = bool_rust_to_mmal(w.custom_background_colour);
        self.inner.custom_background_Y = w.custom_background_y;
        self.inner.custom_background_U = w.custom_background_u;
        self.inner.custom_background_V = w.custom_background_v;
        self.inner.custom_text_colour = bool_rust_to_mmal(w.custom_text_colour);
        self.inner.custom_text_Y = w.custom_text_y;
        self.inner.custom_text_U = w.custom_text_u;
        self.inner.custom_text_V = w.custom_text_v;
        self.inner.text_size = w.text_size;
        
        let sz = (ffi::MMAL_CAMERA_ANNOTATE_MAX_TEXT_LEN_V4 as usize).min(self.inner.text.len());
        let t =  CString::new(w.text.as_bytes()).unwrap_or_default();
        let t = if t.as_bytes_with_nul().len() <= sz { t } else { CString::default() };
        unsafe {
            let src = t.as_bytes_with_nul();
            self.inner.text[0..src.len()].copy_from_slice(std::mem::transmute(src));
        }
        
        self.inner.justify = w.justify;
        self.inner.x_offset = w.x_offset;
        self.inner.y_offset = w.y_offset;
        
    }
}

/// Set the annotate data
pub type PAnnotate = Param<CameraControlPort, AnnotateInnerType>;
