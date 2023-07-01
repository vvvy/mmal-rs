
use std::{io::Write, pin::Pin};
use mmal_rs::{*, ffi::{MMAL_ENCODING_MJPEG, MMAL_ENCODING_H264}};

use log::error;

fn select_camera(print: bool) -> Result<CameraInstanceInfo> {
    let camera_info = CameraInfoComponentHandle::create()?;

    let mut camera_info_param = CameraInformation::default();
    camera_info_param.read(&camera_info)?;
    let w: CameraInfo = camera_info_param.get();

    if print {
        for c in &w.cameras {
            println!("[{}] {} {}x{} lens={}", c.port_id, c.camera_name, c.max_width, c.max_height, c.lens_present)
        }
    }

    Ok(w.cameras.into_iter().next().unwrap())
}

fn wr<R>(l: &str, r: Result<R>) {
    if let Err(e) = r {
        error!("Error[@{l}]: {e}");
    }
}

struct VideoCamera {
    camera: ComponentEnabler<CameraEntity>,
    _encoder: ComponentEnabler<VideoEncoderEntity>,
    connection: ConnectionHandle<CameraVideoPort, VideoEncoderInputPort>,
    encoder_sink: Pin<Box<SinkAggregate<VideoEncoderOutputPort>>>,
}

impl VideoCamera {
    fn create_camera() -> Result<ComponentEnabler<CameraEntity>> {
        let selected_camera = select_camera(true)?;

        let camera = CameraComponentHandle::create()?;
        let camera_num = PCameraNum::from(0);
        let mut ccfg = CameraConfig::from_instance_info(&selected_camera); 
        ccfg.one_shot_stills = false;
        let camera_config = PCameraConfig::from(ccfg);
                /*
                saturation: 50,
                sharpness: 50,
                contrast: 50,
                brightness: 65,
                exposure_compensation: None,
                shutter_speed: 100_000
             */
        CameraControlPort::write_multi(&camera, param_iter![&camera_num, &camera_config])?; 

        CameraControlPort::write(&camera, &PShutterSpeed::from(100_000))?;
        
        let mut vcfg = CAMERA_PORT_CONFIG_320X240;
        vcfg.encoding = ffi::MMAL_ENCODING_I420; //ffi::MMAL_ENCODING_OPAQUE;
        vcfg.encoding_variant = ffi::MMAL_ENCODING_I420;
        vcfg.es_video_frame_rate_num = 10;
        vcfg.es_video_frame_rate_den = 1;
        vcfg.buffer_count_policy = BufferCountPolicy::Recommended;
        CameraVideoPort::configure(&camera, vcfg)?;
        println!("video buffers: {:?}", CameraVideoPort::get_buffers_config(&camera));
        ComponentEnabler::new(camera)
    }

    fn create_encoder(enc: u32) -> Result<ComponentEnabler<VideoEncoderEntity>> {
        let encoder = VideoEncoderComponentHandle::create()?;

        match enc {
            MMAL_ENCODING_MJPEG => {
                let mut format = VideoEncoderOutFormat::default();
                format.encoding = MMAL_ENCODING_MJPEG;
                VideoEncoderOutputPort::configure(&encoder, format)?;
            }
            MMAL_ENCODING_H264 => {
                let mut format = VideoEncoderOutFormat::default();
                format.encoding = MMAL_ENCODING_H264;
                VideoEncoderOutputPort::configure(&encoder, format)?;
        
                let p_video_profile = PVideoProfile::from((VideoProfile::H264Baseline, VideoLevel::H264_4));
                VideoEncoderOutputPort::write(&encoder, &p_video_profile)?;
            }
            _ => panic!("Unsupported encoding")
        }

        println!("encoder buffers: {:?}", VideoEncoderOutputPort::get_buffers_config(&encoder));
        ComponentEnabler::new(encoder)    
    }

    fn create(encoding: u32) -> Result<Self> {
        let camera = Self::create_camera()?;

        let encoder = Self::create_encoder(encoding)?;
    
        let connection = 
            ConnectionHandle::<CameraVideoPort, VideoEncoderInputPort>::create(&camera, &encoder)?;
        connection.enable()?;

        let encoder_sink = SinkAggregate::<VideoEncoderOutputPort>::create(encoder.inner().clone())?;
        encoder_sink.enable()?;

        Ok(Self { camera, _encoder: encoder, connection, encoder_sink })
    } 

    fn stream(&self, output_file: String, max_frames: usize) -> Result<()> {
        let mut file_out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(output_file).unwrap();
        let start = std::time::Instant::now();

        self.encoder_sink.feed_all()?;
        CameraVideoPort::write(&self.camera, &PCaptureVideo::from(true))?;

        let mut count = 0usize;
        let mut total = 0usize;
        let mut last;
        let mut average;
        let mut frame = 0usize;
    
        while let Some(b) = self.encoder_sink.timedwait(5000) {
            let (_, is_terminal) = self.encoder_sink.consume(b, |flags, payload|{
                frame += payload.len();
                file_out.write_all(payload).unwrap();
                Ok((true, flags.is_terminal_frame()))
            })?;
            if is_terminal {
                count += 1;
                total += frame;
                average = total / count;
                last = frame;
                
                frame = 0;
                if count % 100 == 0 {
                    println!("avg={average}(last={last}) total={total}/count={count}");
                }
                if count >= max_frames {
                    break
                } 

            }
        }
        println!("time: {:?}", std::time::Instant::now()-start);
        Ok(())
    }
}

impl Drop for VideoCamera {
    fn drop(&mut self) {
        wr("encoder_sink.disable", self.encoder_sink.disable());
        wr("capture_connection.disable", self.connection.disable());
    }
}

fn main() -> Result<()>{
    env_logger::init();
    mmal_rs::init();

    let mut encoding = MMAL_ENCODING_H264;
    let mut frame_count = 10;
    let mut output_file = "/var/tmp/m.out".to_owned();

    let rmdr: Option<String> = std::env::args().skip(1).fold(None, |s, a| if let Some(s) = s {
        match s.as_ref() {
            "-c" | "--count" => frame_count = a.parse().expect("expected an uint next to --count"),
            "-o" | "--output-file" => output_file = a,
            _ => panic!("invalid command line arg: `{s}`")
        }
        None
    } else {
        match a.as_ref() {
            "--h264" => encoding = MMAL_ENCODING_H264,
            "-m" | "--mjpeg" => encoding = MMAL_ENCODING_MJPEG,
            _ => return Some(a)
        }
        None
    });
    if let Some(w) = rmdr {
        panic!("Invalid command line syntax at EOL: `{w}`")
    }

    let cam = VideoCamera::create(encoding)?;
    cam.stream(output_file, frame_count)

}