//use crate::idp;

use super::*;

//------------------------------------------------------------------------------------------------------------------------------

pub struct VideoEncoderEntity;

impl Entity for VideoEncoderEntity {
    fn name() -> &'static str { "video_encoder" }
}

impl ComponentEntity for VideoEncoderEntity { }

pub type VideoEncoderComponentHandle = ComponentHandle<VideoEncoderEntity>;

impl VideoEncoderComponentHandle {
    pub fn create() -> Result<Self> {
        let component_name: *const c_char = ffi::MMAL_COMPONENT_DEFAULT_VIDEO_ENCODER.as_ptr() as *const c_char;
        unsafe {
            Self::create_from(component_name)
        }
    }

    //unsafe fn input_port(&self)-> *mut ffi::MMAL_PORT_T { self.input_port_n(0) }
    //unsafe fn output_port(&self)-> *mut ffi::MMAL_PORT_T { self.output_port_n(0) }
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct VideoEncoderInputPort;
impl ComponentPort for VideoEncoderInputPort {
    type E = VideoEncoderEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.input_port_n(DEFAULT_PORT_OFFSET)
    }

    fn name() -> &'static str { "video_encoder input port" }
}

pub struct VideoEncoderOutputPort;
impl ComponentPort for VideoEncoderOutputPort {
    type E = VideoEncoderEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.output_port_n(DEFAULT_PORT_OFFSET)
    }

    fn name() -> &'static str { "video_encoder output port" }
}

//------------------------------------------------------------------------------------------------------------------------------
/* 
idp!{MMAL_PARAMETER_JPEG_Q_FACTOR}
///JPEG Quality Factor (default 90)
pub type JpegQFactor = Param<EncoderOutputPort, Uint32<MMAL_PARAMETER_JPEG_Q_FACTOR>>;

idp!{MMAL_PARAMETER_JPEG_RESTART_INTERVAL}
/// JPEG Restart interval (default 0)
pub type JpegRestartInterval = Param<EncoderOutputPort, Uint32<MMAL_PARAMETER_JPEG_RESTART_INTERVAL>>;
*/

/* 
pub struct FpsRangeInnerType {
    inner: ffi::MMAL_PARAMETER_FPS_RANGE_T,
}

impl FpsRangeInnerType {
    pub fn set(&mut self, fps_low: (i32, i32), fps_high: (i32, i32)) {
        self.inner.fps_high 
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

impl Default for FpsRangeInnerType {
    fn default() -> Self { 
        Self { inner: mmal_param_init!(MMAL_PARAMETER_FPS_RANGE_T, MMAL_PARAMETER_FPS_RANGE) }
    }
}

impl InnerParamType for FpsRangeInnerType {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_get(port, &mut self.inner.hdr)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set(port, &self.inner.hdr)
    }

    fn name() -> &'static str { "FpsRangeInnerType" }
}

impl Apply<((i32, i32), (i32, i32))> for FpsRangeInnerType {
    fn apply(&mut self, (xw, yh): ((i32, i32), (i32, i32))) {
        self.set(xw, yh);
    }
}

impl Into<((i32, i32), (i32, i32))> for &'_ FpsRangeInnerType {
    fn into(self) -> ((i32, i32), (i32, i32)) {
        (
            (self.inner.rect.x, self.inner.rect.width),
            (self.inner.rect.y, self.inner.rect.height)
        )
    }
}
*/


pub struct VideoProfileInnerType {
    inner: ffi::MMAL_PARAMETER_VIDEO_PROFILE_T,
}

impl_inner_param_default!{VideoProfileInnerType, MMAL_PARAMETER_VIDEO_PROFILE_T, MMAL_PARAMETER_PROFILE}
impl_inner_param_type!{VideoProfileInnerType}

enumize!{VideoProfile,
    H263Baseline => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_BASELINE,
    H263H320coding => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_H320CODING,
    H263Backwardcompatible => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_BACKWARDCOMPATIBLE,
    H263Iswv2 => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_ISWV2,
    H263Iswv3 => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_ISWV3,
    H263Highcompression => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_HIGHCOMPRESSION,
    H263Internet => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_INTERNET,
    H263Interlace => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_INTERLACE,
    H263Highlatency => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H263_HIGHLATENCY,
    Mp4vSimple => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_SIMPLE,
    Mp4vSimplescalable => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_SIMPLESCALABLE,
    Mp4vCore => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_CORE,
    Mp4vMain => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_MAIN,
    Mp4vNbit => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_NBIT,
    Mp4vScalabletexture => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_SCALABLETEXTURE,
    Mp4vSimpleface => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_SIMPLEFACE,
    Mp4vSimplefba => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_SIMPLEFBA,
    Mp4vBasicanimated => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_BASICANIMATED,
    Mp4vHybrid => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_HYBRID,
    Mp4vAdvancedrealtime => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_ADVANCEDREALTIME,
    Mp4vCorescalable => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_CORESCALABLE,
    Mp4vAdvancedcoding => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_ADVANCEDCODING,
    Mp4vAdvancedcore => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_ADVANCEDCORE,
    Mp4vAdvancedscalable => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_ADVANCEDSCALABLE,
    Mp4vAdvancedsimple => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_MP4V_ADVANCEDSIMPLE,
    H264Baseline => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_BASELINE,
    H264Main => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_MAIN,
    H264Extended => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_EXTENDED,
    H264High => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_HIGH,
    H264High10 => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_HIGH10,
    H264High422 => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_HIGH422,
    H264High444 => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_HIGH444,
    H264ConstrainedBaseline => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_H264_CONSTRAINED_BASELINE,
    Dummy => MMAL_VIDEO_PROFILE_T_MMAL_VIDEO_PROFILE_DUMMY
}

enumize!{VideoLevel,
    H263_10 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_10,
    H263_20 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_20,
    H263_30 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_30,
    H263_40 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_40,
    H263_45 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_45,
    H263_50 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_50,
    H263_60 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_60,
    H263_70 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H263_70,
    MP4V_0 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_0,
    MP4V_0b => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_0b,
    MP4V_1 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_1,
    MP4V_2 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_2,
    MP4V_3 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_3,
    MP4V_4 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_4,
    MP4V_4a => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_4a,
    MP4V_5 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_5,
    MP4V_6 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_MP4V_6,
    H264_1 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_1,
    H264_1b => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_1b,
    H264_11 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_11,
    H264_12 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_12,
    H264_13 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_13,
    H264_2 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_2,
    H264_21 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_21,
    H264_22 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_22,
    H264_3 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_3,
    H264_31 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_31,
    H264_32 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_32,
    H264_4 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_4,
    H264_41 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_41,
    H264_42 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_42,
    H264_5 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_5,
    H264_51 => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_H264_51,
    Dummy => MMAL_VIDEO_LEVEL_T_MMAL_VIDEO_LEVEL_DUMMY
}


impl VideoProfileInnerType {
    pub fn set(&mut self, profile: VideoProfile, level: VideoLevel) {
        self.inner.profile[0].profile = profile as u32;
        self.inner.profile[0].level = level as u32;
    }

    pub fn new(profile: VideoProfile, level: VideoLevel) -> Self {
        let mut rv = Self::default();
        rv.set(profile, level);
        rv
    }
}

impl Apply<(VideoProfile, VideoLevel)> for VideoProfileInnerType {
    fn apply(&mut self, (profile, level): (VideoProfile, VideoLevel)) {
        self.set(profile, level);
    }
}

impl TryInto<(VideoProfile, VideoLevel)> for &'_ VideoProfileInnerType {
    fn try_into(self) -> Result<(VideoProfile, VideoLevel)> {
        Ok((self.inner.profile[0].profile.try_into()?, self.inner.profile[0].level.try_into()?))
    }

    type Error = MmalError;
}

/// Video profile
pub type PVideoProfile = Param<VideoEncoderOutputPort, VideoProfileInnerType>;

//------------------------------------------------------------------------------------------------------------------------------
pub struct VideoEncoderOutFormat {
    pub encoding: u32,
    pub bitrate: u32
}

impl Default for VideoEncoderOutFormat {
    fn default() -> Self { Self { 
        encoding: ffi::MMAL_ENCODING_H264,
        bitrate: 300_000
    } }
}


impl PortConfig for VideoEncoderOutFormat {
    unsafe fn apply_format(&self, port: *mut ffi::MMAL_PORT_T) {
        let format = &mut (*(*port).format);
        format.encoding = self.encoding;
        format.bitrate = self.bitrate;
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
