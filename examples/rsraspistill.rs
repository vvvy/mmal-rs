
use std::{io::Write, pin::Pin, path::Path};
use mmal_rs::*;

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



struct StillCamera {
    camera: ComponentEnabler<CameraEntity>,
    _encoder: ComponentEnabler<EncoderEntity>,
    connection: ConnectionHandle<CameraCapturePort, EncoderInputPort>,
    encoder_sink: Pin<Box<SinkAggregate<EncoderOutputPort>>>,
}

impl StillCamera {

    fn create_camera(selected_camera: &CameraInstanceInfo) -> Result<ComponentEnabler<CameraEntity>> {
        let camera = CameraComponentHandle::create()?;
        let camera_num = PCameraNum::from(0);
        let camera_config = PCameraConfig::from(CameraConfig::from_instance_info(selected_camera));
        let camera_shutter_speed = PShutterSpeed::from(100_000);
                /*
                saturation: 50,
                sharpness: 50,
                contrast: 50,
                brightness: 65,
                exposure_compensation: None,
                shutter_speed: 100_000
             */
        let mut annotate = Annotate::default();
        annotate.show_analog_gain = true;
        annotate.show_caf = true;
        annotate.show_shutter = true;
        annotate.text = "VyborCam".into();
        let annotate_p = PAnnotate::from(&annotate);

        CameraControlPort::write_multi(&camera, 
            param_iter![&camera_num, &camera_config, &camera_shutter_speed, &annotate_p])?;    
        CameraCapturePort::configure(&camera, CAMERA_PORT_CONFIG_320X240)?;
        ComponentEnabler::new(camera)
    }

    fn create_encoder() -> Result<ComponentEnabler<EncoderEntity>> {
        let encoder = EncoderComponentHandle::create()?;
    
        EncoderOutputPort::configure(&encoder, EncoderOutFormat::default())?;
        EncoderOutputPort::write(&encoder, &JpegQFactor::from(90))?;
        EncoderOutputPort::write(&encoder, &JpegRestartInterval::from(0))?;
        ComponentEnabler::new(encoder)
    }

    fn create() -> Result<Self> {
        let selected_camera = select_camera(true)?;

        let camera = Self::create_camera(&selected_camera)?;
        let encoder = Self::create_encoder()?;

        let connection = 
            ConnectionHandle::<CameraCapturePort, EncoderInputPort>::create(&camera, &encoder)?;
        connection.enable()?;

        let encoder_sink = SinkAggregate::<EncoderOutputPort>::create(encoder.as_ref().clone())?;
        encoder_sink.enable()?;

        Ok(Self { camera, _encoder: encoder, connection, encoder_sink })
    } 

    fn take_one_shot<P: AsRef<Path>>(&self, output_file: P) -> Result<()> {

        let mut file_out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(output_file).unwrap();

        let start = std::time::Instant::now();

        self.encoder_sink.feed_all()?;
        CameraCapturePort::write(self.camera.as_ref(), &PCapture::from(true))?;
    
        while let Some(b) = self.encoder_sink.timedwait(5000) {
            let (_, is_terminal) = self.encoder_sink.consume(b, |flags, payload|{
                println!("rec'd {}", payload.len());
                file_out.write_all(payload).unwrap();
                Ok((true, flags.is_terminal_frame()))
            })?;
            if is_terminal { break }
        }
        println!("time: {:?}", std::time::Instant::now()-start);

        Ok(())
    }
}

impl Drop for StillCamera {
    fn drop(&mut self) {
        wr("encoder_sink.disable", self.encoder_sink.disable());
        wr("connection.disable", self.connection.disable());
    }
}


struct VideoCamera {
    camera: ComponentEnabler<CameraEntity>,
    _encoder: ComponentEnabler<EncoderEntity>,
    connection: ConnectionHandle<CameraVideoPort, EncoderInputPort>,
    encoder_sink: Pin<Box<SinkAggregate<EncoderOutputPort>>>,
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
        vcfg.encoding = ffi::MMAL_ENCODING_I420;
        vcfg.encoding_variant = ffi::MMAL_ENCODING_I420;
        vcfg.es_video_frame_rate_num = 3;
        vcfg.es_video_frame_rate_den = 1;
        vcfg.buffer_count_policy = BufferCountPolicy::Explicit(3);
        CameraVideoPort::configure(&camera, vcfg)?;
        println!("video buffers: {:?}", CameraVideoPort::get_buffers_config(&camera));
        ComponentEnabler::new(camera)
    }

    fn create_encoder() -> Result<ComponentEnabler<EncoderEntity>> {
        let encoder = EncoderComponentHandle::create()?;
    
        EncoderOutputPort::configure(&encoder, EncoderOutFormat::default())?;
        EncoderOutputPort::write(&encoder, &JpegQFactor::from(90))?;
        EncoderOutputPort::write(&encoder, &JpegRestartInterval::from(0))?;
        println!("encoder buffers: {:?}", EncoderOutputPort::get_buffers_config(&encoder));
        ComponentEnabler::new(encoder)    
    }

    fn create() -> Result<Self> {
        let camera = Self::create_camera()?;

        let encoder = Self::create_encoder()?;
    
        let connection = 
            ConnectionHandle::<CameraVideoPort, EncoderInputPort>::create(&camera, &encoder)?;
        connection.enable()?;

        let encoder_sink = SinkAggregate::<EncoderOutputPort>::create(encoder.inner().clone())?;
        encoder_sink.enable()?;

        Ok(Self { camera, _encoder: encoder, connection, encoder_sink })
    } 


    fn add_file_number(f: &str, n: usize) -> String {
        let mut i = f.rsplitn(2, '.');
        let last = i.next().expect("invalid file name");
        if let Some(first) = i.next() {
            format!("{first}{n}.{last}")
        } else {
            format!("{last}{n}")
        }
    }

    fn stream(&self, output_file: String, stills_count: usize) -> Result<()> {

        let mut still_n = 0;
        let mut file_out = None;
        let start = std::time::Instant::now();

        self.encoder_sink.feed_all()?;
        CameraVideoPort::write(&self.camera, &PCaptureVideo::from(true))?;
    
        while let Some(b) = self.encoder_sink.timedwait(5000) {
            if file_out.is_none() {
                //let output_file_split = Path::file_name(&self) output_file.split();
                let ofn = Self::add_file_number(&output_file, still_n);
                file_out = Some(std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(ofn).unwrap());
            }
            let (_, is_terminal) = self.encoder_sink.consume(b, |flags, payload|{
                println!("rec'd {}", payload.len());
                file_out.as_mut().unwrap().write_all(payload).unwrap();
                Ok((true, flags.is_terminal_frame()))
            })?;
            if is_terminal {
                println!("time: {:?}", std::time::Instant::now()-start);
                still_n += 1;
                if still_n >= stills_count { break }
                file_out = None;
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


#[test]
fn test_add_fn() {
    assert_eq!("/abc/d/e5.x", VideoCamera::add_file_number("/abc/d/e.x", 5));
    assert_eq!("/abc/d/e5", VideoCamera::add_file_number("/abc/d/e", 5));
    assert_eq!("e10", VideoCamera::add_file_number("e", 10));
}

fn main() -> Result<()>{
    env_logger::init();
    mmal_rs::init();

    let mut use_video = false;
    let mut stills_count = 10;
    let mut output_file = "/var/tmp/f.jpg".to_owned();

    let rmdr: Option<String> = std::env::args().skip(1).fold(None, |s, a| if let Some(s) = s {
        match s.as_ref() {
            "-c" | "--count" => stills_count = a.parse().expect("expected an uint next to --count"),
            "-o" | "--output-file" => output_file = a,
            _ => panic!("invalid command line arg: `{s}`")
        }
        None
    } else {
        match a.as_ref() {
            "-v" | "--video" => use_video = true,
            _ => return Some(a)
        }
        None
    });
    if let Some(w) = rmdr {
        panic!("Invalid command line syntax at EOL: `{w}`")
    }

    println!("use_video={use_video}");

    if use_video {
        let cam = VideoCamera::create()?;
        cam.stream(output_file, stills_count)
    } else {
        let cam = StillCamera::create()?;
        cam.take_one_shot(output_file)
    }
}