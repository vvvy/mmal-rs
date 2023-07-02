
use std::{pin::Pin, sync::Arc, fmt};
use mmal_rs::{*, ffi::MMAL_ENCODING_MJPEG};
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast, io::{AsyncReadExt,AsyncWriteExt}};

use log::{debug, info, error};

#[derive(Debug)]
pub enum Error {
    Mmal(MmalError),
    Io(std::io::Error)
}

impl From<MmalError> for Error {
    fn from(value: MmalError) -> Self {
        Self::Mmal(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mmal(e) => write!(f, "mmal: {e}"),
            Self::Io(e) => write!(f, "IO: {e}"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

struct Settings {
    max_frame_count: usize,
    initial_frame_buffer_len: usize,
    client_hs_buffer_len: usize, 
    stats_period: usize,
    bind_addr: String,
    bitrate: u32,
    frame_rate: i32,
    shutter_speed: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { 
            max_frame_count: 0, 
            initial_frame_buffer_len: 100_000, 
            client_hs_buffer_len: 1_000,
            stats_period: 1_000, 
            bind_addr: "0.0.0.0:9990".to_owned(),
            bitrate: 1_000_000,
            frame_rate: 25,
            shutter_speed: 40_000
        }
    }
}


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

fn wr<R>(l: &str, r: mmal_rs::Result<R>) {
    if let Err(e) = r {
        error!("Error[@{l}]: {e}");
    }
}

struct VideoCamera {
    camera: ComponentEnabler<CameraEntity>,
    _encoder: ComponentEnabler<VideoEncoderEntity>,
    connection: ConnectionHandle<CameraVideoPort, VideoEncoderInputPort>,
    encoder_sink: Pin<Box<SinkAggregate<VideoEncoderOutputPort>>>,
    settings: Settings,
}

impl VideoCamera {
    fn create_camera(settings: &Settings) -> Result<ComponentEnabler<CameraEntity>> {
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
             */
        CameraControlPort::write_multi(&camera, param_iter![&camera_num, &camera_config])?; 

        CameraControlPort::write(&camera, &PShutterSpeed::from(settings.shutter_speed))?;
        
        let mut vcfg = CAMERA_PORT_CONFIG_320X240;
        vcfg.encoding = ffi::MMAL_ENCODING_I420; //ffi::MMAL_ENCODING_OPAQUE;
        vcfg.encoding_variant = ffi::MMAL_ENCODING_I420;
        vcfg.es_video_frame_rate_num = settings.frame_rate;
        vcfg.es_video_frame_rate_den = 1;
        vcfg.buffer_count_policy = BufferCountPolicy::Recommended;
        CameraVideoPort::configure(&camera, vcfg)?;
        println!("video buffers: {:?}", CameraVideoPort::get_buffers_config(&camera));
        Ok(ComponentEnabler::new(camera)?)
    }

    fn create_encoder(settings: &Settings) -> Result<ComponentEnabler<VideoEncoderEntity>> {
        let encoder = VideoEncoderComponentHandle::create()?;
        let mut format = VideoEncoderOutFormat::default();
        format.encoding = MMAL_ENCODING_MJPEG;
        format.bitrate = settings.bitrate;
        VideoEncoderOutputPort::configure(&encoder, format)?;
        println!("encoder buffers: {:?}", VideoEncoderOutputPort::get_buffers_config(&encoder));
        Ok(ComponentEnabler::new(encoder)?)
    }

    fn create(settings: Settings) -> Result<Self> {
        let camera = Self::create_camera(&settings)?;

        let encoder = Self::create_encoder(&settings)?;
    
        let connection = 
            ConnectionHandle::<CameraVideoPort, VideoEncoderInputPort>::create(&camera, &encoder)?;
        connection.enable()?;

        let encoder_sink = SinkAggregate::<VideoEncoderOutputPort>::create(encoder.inner().clone())?;
        encoder_sink.enable()?;

        Ok(Self { camera, _encoder: encoder, connection, encoder_sink, settings })
    }

    async fn run_server(mut stream: TcpStream, mut rx: broadcast::Receiver<Arc<Vec<u8>>>, client_hs_buffer_len: usize) -> Result<()> {
        let mut client_hs = vec![0u8; client_hs_buffer_len];
        let client_hs_len = stream.read(&mut client_hs).await?;
        debug!("client hs: {}", String::from_utf8_lossy(&client_hs[0..client_hs_len]));

        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace;boundary=MJPEGBOUNDARY\r\n").await?;

        loop {
            let payload = match rx.recv().await {
                Ok(payload) => payload,
                Err(broadcast::error::RecvError::Closed) => {
                    info!("{}: disconnect", stream.peer_addr()?);
                    return Ok(());
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    debug!("{}: lagged by {}", stream.peer_addr()?, n);
                    continue;
                }
            };

            let header = format!("\r\n--MJPEGBOUNDARY\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nX-Timestamp: 0.000000\r\n\r\n", payload.len());
            stream.write_all(header.as_bytes()).await?;
            stream.write_all(payload.as_ref()).await?;
        }
    }

    async fn stream(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.settings.bind_addr).await?;
        let (tx, _) = broadcast::channel(3);

        let mut count = 0usize;
        let mut total = 0usize;
        let mut last;
        let mut average;

        let mut frame = Vec::with_capacity(self.settings.initial_frame_buffer_len);

        self.encoder_sink.feed_all()?;
        CameraVideoPort::write(&self.camera, &PCaptureVideo::from(true))?;

        loop {
            tokio::select! {
                sa = listener.accept() => {
                    let (stream, addr) = sa?;
                    info!("Connection from {}", addr);
                    let rx = tx.subscribe();
                    let client_hs_buffer_len = self.settings.client_hs_buffer_len;
                    tokio::spawn(async move { if let Err(e) = Self::run_server(stream, rx, client_hs_buffer_len).await {
                        error!("Server error: {}", e);
                    } });
                }
            
                buffer = &mut self.encoder_sink => {
                    let (_, is_terminal) = self.encoder_sink.consume(buffer, |flags, payload|{
                        frame.extend_from_slice(payload);
                        Ok((true, flags.is_terminal_frame()))
                    })?;
                    if is_terminal {
                        count += 1;
                        if self.settings.stats_period > 0 {
                            last = frame.len();

                            total += last;
                            average = total / count;
                            if count % self.settings.stats_period == 0 {
                                let clients = tx.receiver_count();
                                debug!("avg={average}(last={last}) total={total}/count={count} clients={clients}");
                            }
                        }

                        if self.settings.max_frame_count > 0 && count >= self.settings.max_frame_count {
                            break
                        }
                        let mut payload = Vec::with_capacity(self.settings.initial_frame_buffer_len);
                        std::mem::swap(&mut payload, &mut frame);
                        let _ = tx.send(Arc::new(payload));                     
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for VideoCamera {
    fn drop(&mut self) {
        wr("encoder_sink.disable", self.encoder_sink.disable());
        wr("capture_connection.disable", self.connection.disable());
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()>{
    env_logger::init();
    mmal_rs::init();

    let mut settings = Settings::default();

    //TODO add to parameters:
    //- saturation,..(?)
    //- pic size
    //TODO try to reduce max pic size in camera settings (target: lag=0)

    let rmdr: Option<String> = std::env::args().skip(1).fold(None, |s, a| if let Some(s) = s {
        match s.as_ref() {
            "-c" | "--count" => settings.max_frame_count = a.parse().expect("expected an uint next to --count"),
            "-b" | "--bind-addr" => settings.bind_addr = a,
            "-f" | "--frame-rate" => settings.frame_rate = a.parse().expect("expected an uint next to --frame-rate"),
            "-B" | "--bitrate" => settings.bitrate = a.parse().expect("expected an uint next to --bitrate"),
            "-s" | "--shutter-speed" => settings.shutter_speed = a.parse().expect("expected an uint next to --shutter-speed"),
            _ => panic!("invalid command line arg: `{s}`")
        }
        None
    } else {
        match a.as_ref() {
            "-h" | "--help" => {
                let ss = Settings::default();
                println!(r#"
Usage: mjpeg-streamer [options...]

Options are:
--count|-c <uint>               Exit after streaming this number of frames. 0 to stream infinitely (default {c})
--bind-addr|-b <addr>           Socket address to bind to (default {b})
--frame-rate|-f <uint>          Camera frame rate (default {f})
--bitrate|-B <uint>             Encoder output bitrate (default {B})
--shutter-speed|-s <uint>       Camera shutter speed in microseconds (default {s})
--help|-h                       Print this help message and exit
"#,
b=ss.bind_addr,
c=ss.max_frame_count,
f=ss.frame_rate,
B=ss.bitrate,
s=ss.shutter_speed,
);
                std::process::exit(0);
            }
            _ => return Some(a)
        }
        //None
    });
    if let Some(w) = rmdr {
        panic!("Invalid command line syntax at EOL: `{w}`")
    }

    let mut cam = VideoCamera::create(settings)?;
    cam.stream().await

}