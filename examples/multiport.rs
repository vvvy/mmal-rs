
use std::io::Write;
use mmal_rs::*;

use log::{trace, error};

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



pub fn service() -> Result<()> {
    let selected_camera = select_camera(true)?;

    let camera = CameraComponentHandle::create()?;

    let encoder = EncoderComponentHandle::create()?;

    let preview = NullSinkComponentHandle::create()?;

    trace!("point 1");


    let camera_num = PCameraNum::from(0);
    let camera_config = PCameraConfig::from_update(CameraConfig::from_instance_info(&selected_camera));
    let camera_shutter_speed = PShutterSpeed::from(100_000);
            /*
            saturation: 50,
            sharpness: 50,
            contrast: 50,
            brightness: 65,
            exposure_compensation: None,
            shutter_speed: 100_000
         */
    CameraControlPort::write_multi(&camera, 
        param_iter![&camera_num, &camera_config, &camera_shutter_speed])?;

    trace!("point 2");

    CameraCapturePort::configure(&camera, CAMERA_PORT_CONFIG_320X240)?;
    CameraPreviewPort::configure(&camera, CAMERA_PORT_CONFIG_320X240)?;
    CameraVideoPort::configure(&camera,CAMERA_PORT_CONFIG_320X240)?;

    EncoderOutputPort::configure(&encoder, EncoderOutFormat::default())?;
    EncoderOutputPort::write(&encoder, &JpegQFactor::from(90))?;
    EncoderOutputPort::write(&encoder, &JpegRestartInterval::from(0))?;

    trace!("point 3");

    let capture_connection = 
        ConnectionHandle::<CameraCapturePort, EncoderInputPort>::create(&camera, &encoder)?;

    capture_connection.enable()?;

    let preview_connection = 
        ConnectionHandle::<CameraPreviewPort, NullSinkInputPort>::create(&camera, &preview)?;

    preview_connection.enable()?;

    println!("point 4");

    let encoder_sink = SinkAggregate::<EncoderOutputPort>::create(encoder)?;
    trace!("point 4.1");
    encoder_sink.enable()?;
    trace!("point 4.2");
    encoder_sink.feed_all()?;
    trace!("point 4.3");

    camera.enable()?;

    trace!("point 5");

    let mut file_out = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open("/var/tmp/f.jpg").unwrap();

    trace!("point 6");

    CameraCapturePort::write(&camera, &PCapture::from(true))?;

    trace!("point 7");

    while let Some(b) = encoder_sink.timedwait(5000) {
        let (_, is_terminal) = encoder_sink.consume(b, |flags, payload|{
            println!("rec'd {}", payload.len());
            file_out.write_all(payload).unwrap();
            Ok((true, flags.is_terminal_frame()))
        })?;
        if is_terminal { break }
    }
    
    wr("camera.disable", camera.disable());
    wr("encoder_sink.disable", encoder_sink.disable());
    wr("preview_connection.disable", preview_connection.disable());
    wr("capture_connection.disable", capture_connection.disable());

    Ok(())
}




fn main() -> Result<()>{
    env_logger::init();
    mmal_rs::init();
    service()
}