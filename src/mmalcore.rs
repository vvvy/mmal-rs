use std::{ptr::NonNull, mem::MaybeUninit, marker::{PhantomData, PhantomPinned}, ffi::c_char, sync::Once, pin::Pin};
use std::task::{Poll, Context, Waker};
use super::*;


//------------------------------------------------------------------------------------------------------------------------------
//------------------------------------------------------------------------------------------------------------------------------


/// Common companion trait for all MMAL object types
pub trait Entity {
    fn name() -> &'static str;
}

/// Component companion trait
pub trait ComponentEntity: Entity { }

//------------------------------------------------------------------------------------------------------------------------------

/// MMAL Component Handle
/// 
/// This is essentially a C-stype pointer with a companion type attached. Note that cloning the handle
/// effectivelly copies the pointer and increases refcount of the underlying C struct. 
pub struct ComponentHandle<E: ComponentEntity> {
    c: NonNull<ffi::MMAL_COMPONENT_T>,
    t: PhantomData<E>
}

impl<E: ComponentEntity> ComponentHandle<E> {
    /// Creates a new component instance and returns a handle to it
    pub(super) unsafe fn create_from(component_name: *const c_char) -> Result<Self> {
        let mut ptr = MaybeUninit::uninit();
        let status = ffi::mmal_component_create(component_name, ptr.as_mut_ptr());
        cst!(status, "{}: Unable to create component", E::name())?;
        let ptr: *mut ffi::MMAL_COMPONENT_T = ptr.assume_init();
        let c = NonNull::new(ptr).unwrap();
        Ok(Self { c, t: PhantomData })
    }

    pub fn enable(&self) -> Result<()> {
        unsafe {
            if self.c.as_ref().is_enabled == 0 {
                let status = ffi::mmal_component_enable(self.c.as_ptr());
                cst!(status, "{}: unable to enable", E::name())?;
            }
        }
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        unsafe {
            if self.c.as_ref().is_enabled != 0 {
                let status = ffi::mmal_component_disable(self.c.as_ptr());
                cst!(status, "{}: unable to disable", E::name())?;
            }
        }
        Ok(())
    }

    pub(super) unsafe fn control_port(&self) -> *mut ffi::MMAL_PORT_T { self.c.as_ref().control }
    pub(super) unsafe fn output_port_n(&self, n: isize) -> *mut ffi::MMAL_PORT_T { 
        assert!(n < self.c.as_ref().output_num as isize, "invalid output port {} (total ports {})", n, self.c.as_ref().output_num);
        *self.c.as_ref().output.offset(n)
    }
    pub(super) unsafe fn input_port_n(&self, n: isize) -> *mut ffi::MMAL_PORT_T { 
        assert!(n < self.c.as_ref().input_num as isize, "invalid input port {} (total ports {})", n, self.c.as_ref().input_num);
        *self.c.as_ref().input.offset(n) 
    }

    /*pub fn configure<'p>(&self, settings: impl Iterator<Item=&'p E::ComponentParam>) -> Result<()> {
        unsafe { E::configure(self, settings) }
    }

    pub fn read_config<'p>(&self, settings: impl Iterator<Item=&'p mut E::ComponentParam>) -> Result<()> {
        unsafe { E::read_config(self, settings) }
    }*/
}

impl<E: ComponentEntity> Drop for ComponentHandle<E> {
    fn drop(&mut self) {
        unsafe { log_deinit!(cst!(ffi::mmal_component_destroy(self.c.as_ptr()), "mmal_component_destroy({})", E::name())); }
    }
}

impl<E: ComponentEntity> Clone for ComponentHandle<E> {
    fn clone(&self) -> Self {
        unsafe { ffi::mmal_component_acquire(self.c.as_ptr()) }
        Self { c: self.c.clone(), t: self.t.clone() }
    }
}

impl<E: ComponentEntity> AsRef<ComponentHandle<E>> for ComponentHandle<E> {
    fn as_ref(&self) -> &ComponentHandle<E> { self }
}


//------------------------------------------------------------------------------------------------------------------------------

/// A component handle container that allows 'automatic' enabling a component during the container's lifespan
pub struct ComponentEnabler<E: ComponentEntity> {
    h: ComponentHandle<E>
}

impl<E: ComponentEntity> ComponentEnabler<E> {
    pub fn new(h: ComponentHandle<E>) -> Result<Self> {
        h.enable()?;
        Ok(Self { h })
    }

    pub fn with_init(h: ComponentHandle<E>, f: impl FnOnce(&ComponentHandle<E>) -> Result<()>) -> Result<Self> {
        f(&h)?;
        h.enable()?;
        Ok(Self { h })
    }

    pub fn inner(&self) -> &ComponentHandle<E> { 
        &self.h 
    }
}

impl<E: ComponentEntity> AsRef<ComponentHandle<E>> for ComponentEnabler<E> {
    fn as_ref(&self) -> &ComponentHandle<E> {
        &self.h
    }
}

impl<E: ComponentEntity> AsMut<ComponentHandle<E>> for ComponentEnabler<E> {
    fn as_mut(&mut self) -> &mut ComponentHandle<E> {
        &mut self.h
    }
}

impl<E: ComponentEntity> Drop for ComponentEnabler<E> {
    fn drop(&mut self) {
        log_deinit!(self.h.disable());
    }
}


//------------------------------------------------------------------------------------------------------------------------------

pub struct NullSinkEntity;

impl Entity for NullSinkEntity {
    fn name() -> &'static str { "null_sink" }
}

impl ComponentEntity for NullSinkEntity { }

pub type NullSinkComponentHandle = ComponentHandle<NullSinkEntity>;


impl NullSinkComponentHandle {
    pub fn create() -> Result<Self> {
        let component_name: *const c_char = ffi::MMAL_COMPONENT_NULL_SINK.as_ptr() as *const std::ffi::c_char;
        unsafe {
            Self::create_from(component_name)
        }
    }

    //pub fn input_port(&self)-> *mut ffi::MMAL_PORT_T { unsafe { self.input_port_n(DEFAULT_PORT_OFFSET) } }
}

pub struct NullSinkInputPort;
impl ComponentPort for NullSinkInputPort {
    type E = NullSinkEntity;

    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T {
        component.input_port_n(DEFAULT_PORT_OFFSET)
    }

    fn name() -> &'static str { "null sink input port" }    
}

//------------------------------------------------------------------------------------------------------------------------------

pub struct ConnectionHandle<PS: ComponentPort, PT: ComponentPort> {
    c: NonNull<ffi::MMAL_CONNECTION_T>,
    source: PhantomData<PS>,
    target: PhantomData<PT>,
}

impl<PS: ComponentPort, PT: ComponentPort> ConnectionHandle<PS, PT> {
    pub fn create(source: impl AsRef<ComponentHandle<PS::E>>, target: impl AsRef<ComponentHandle<PT::E>>) -> Result<Self> {
        let c = unsafe {     
            let mut connection_ptr = MaybeUninit::uninit();
            let status = ffi::mmal_connection_create(
                connection_ptr.as_mut_ptr(),
                PS::get_port(source.as_ref()),
                PT::get_port(target.as_ref()),
                ffi::MMAL_CONNECTION_FLAG_TUNNELLING
                    | ffi::MMAL_CONNECTION_FLAG_ALLOCATION_ON_INPUT,
            );
            cst!(status, "unable to create connection {}->{}", PS::name(), PT::name())?;
            let connection_ptr: *mut ffi::MMAL_CONNECTION_T = connection_ptr.assume_init();
            NonNull::new(connection_ptr).unwrap()         
        };
        Ok(Self{ c, source: PhantomData, target: PhantomData })
    }

    pub fn enable(&self) -> Result<()> {
        let status = unsafe { ffi::mmal_connection_enable(self.c.as_ptr()) };
        cst!(status, "unable to enable connection {}->{}", PS::name(), PT::name())?;
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let status = unsafe { ffi::mmal_connection_disable(self.c.as_ptr()) };
        cst!(status, "unable to disable connection {}->{}", PS::name(), PT::name())?;
        Ok(())
    }
}

impl<PS: ComponentPort, PT: ComponentPort> Drop for ConnectionHandle<PS, PT> {
    fn drop(&mut self) {
        unsafe {
            log_deinit!(cst!(ffi::mmal_connection_destroy(self.c.as_ptr()), "mmal_connection_destroy({}->{})", PS::name(), PT::name()));
        }
    }
}

impl<PS: ComponentPort, PT: ComponentPort> Clone for ConnectionHandle<PS, PT> {
    fn clone(&self) -> Self {
        unsafe { ffi::mmal_connection_acquire(self.c.as_ptr()) }
        Self { c: self.c.clone(), source: self.source.clone(), target: self.target.clone() }
    }
}


//------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct FrameFlags {
    flags: u32
}

impl FrameFlags {
    pub const FLAG_EOS: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_EOS;  /// MMAL_BUFFER_HEADER_FLAG_EOS: u32 = 1;
    pub const FLAG_FRAME_START: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_FRAME_START;  /// MMAL_BUFFER_HEADER_FLAG_FRAME_START: u32 = 2;
    pub const FLAG_FRAME_END: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_FRAME_END;  /// MMAL_BUFFER_HEADER_FLAG_FRAME_END: u32 = 4;
    pub const FLAG_FRAME: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_FRAME;  /// MMAL_BUFFER_HEADER_FLAG_FRAME: u32 = 6;
    pub const FLAG_KEYFRAME: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_KEYFRAME;  /// MMAL_BUFFER_HEADER_FLAG_KEYFRAME: u32 = 8;
    pub const FLAG_DISCONTINUITY: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_DISCONTINUITY;  /// MMAL_BUFFER_HEADER_FLAG_DISCONTINUITY: u32 = 16;
    pub const FLAG_CONFIG: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_CONFIG;  /// MMAL_BUFFER_HEADER_FLAG_CONFIG: u32 = 32;
    pub const FLAG_ENCRYPTED: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_ENCRYPTED;  /// MMAL_BUFFER_HEADER_FLAG_ENCRYPTED: u32 = 64;
    pub const FLAG_CODECSIDEINFO: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_CODECSIDEINFO;  /// MMAL_BUFFER_HEADER_FLAG_CODECSIDEINFO: u32 = 128;
    pub const FLAGS_SNAPSHOT: u32 = ffi::MMAL_BUFFER_HEADER_FLAGS_SNAPSHOT;  /// MMAL_BUFFER_HEADER_FLAGS_SNAPSHOT: u32 = 256;
    pub const FLAG_CORRUPTED: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_CORRUPTED;  /// MMAL_BUFFER_HEADER_FLAG_CORRUPTED: u32 = 512;
    pub const FLAG_TRANSMISSION_FAILED: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_TRANSMISSION_FAILED;  /// MMAL_BUFFER_HEADER_FLAG_TRANSMISSION_FAILED: u32 = 1024;
    pub const FLAG_DECODEONLY: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_DECODEONLY;  /// MMAL_BUFFER_HEADER_FLAG_DECODEONLY: u32 = 2048;
    pub const FLAG_NAL_END: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_NAL_END;  /// MMAL_BUFFER_HEADER_FLAG_NAL_END: u32 = 4096;
    pub const FLAG_USER0: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_USER0;  /// MMAL_BUFFER_HEADER_FLAG_USER0: u32 = 268435456;
    pub const FLAG_USER1: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_USER1;  /// MMAL_BUFFER_HEADER_FLAG_USER1: u32 = 536870912;
    pub const FLAG_USER2: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_USER2;  /// MMAL_BUFFER_HEADER_FLAG_USER2: u32 = 1073741824;
    pub const FLAG_USER3: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_USER3;  /// MMAL_BUFFER_HEADER_FLAG_USER3: u32 = 2147483648;
    pub const FLAG_FORMAT_SPECIFIC_START_BIT: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_FORMAT_SPECIFIC_START_BIT;  /// MMAL_BUFFER_HEADER_FLAG_FORMAT_SPECIFIC_START_BIT: u32 = 16;
    pub const FLAG_FORMAT_SPECIFIC_START: u32 = ffi::MMAL_BUFFER_HEADER_FLAG_FORMAT_SPECIFIC_START;  /// MMAL_BUFFER_HEADER_FLAG_FORMAT_SPECIFIC_START: u32 = 65536;
    pub const VIDEO_FLAG_INTERLACED: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_INTERLACED;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_INTERLACED: u32 = 65536;
    pub const VIDEO_FLAG_TOP_FIELD_FIRST: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_TOP_FIELD_FIRST;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_TOP_FIELD_FIRST: u32 = 131072;
    pub const VIDEO_FLAG_DISPLAY_EXTERNAL: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_DISPLAY_EXTERNAL;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_DISPLAY_EXTERNAL: u32 = 524288;
    pub const VIDEO_FLAG_PROTECTED: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_PROTECTED;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_PROTECTED: u32 = 1048576;
    pub const VIDEO_FLAG_COLUMN_LOG2_SHIFT: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_COLUMN_LOG2_SHIFT;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_COLUMN_LOG2_SHIFT: u32 = 24;
    pub const VIDEO_FLAG_COLUMN_LOG2_MASK: u32 = ffi::MMAL_BUFFER_HEADER_VIDEO_FLAG_COLUMN_LOG2_MASK;  /// MMAL_BUFFER_HEADER_VIDEO_FLAG_COLUMN_LOG2_MASK: u32 = 251658240;

    pub const FLAG_TERMINAL_FRAME: u32 = Self::FLAG_FRAME_END | Self::FLAG_TRANSMISSION_FAILED;

    pub fn test_one(&self, mask: u32) -> bool { self.flags & mask != 0 }
    pub fn test_all(&self, mask: u32) -> bool { self.flags & mask == mask }
    pub fn is_terminal_frame(&self) -> bool { self.test_one(Self::FLAG_TERMINAL_FRAME) }
}


//------------------------------------------------------------------------------------------------------------------------------


pub struct BufferRef {
    p: NonNull<ffi::MMAL_BUFFER_HEADER_T>,
}

impl BufferRef {


    /// Returns (is_consumed, user_data). If !is_consumed, should be pushed back to the queue.
    pub fn do_locked<R>(&self, mut f: impl FnMut(FrameFlags, &[u8]) -> Result<(bool, R)>) -> Result<(bool, R)> {
        unsafe {
            let flags = FrameFlags { flags: self.p.as_ref().flags };
            cst!(ffi::mmal_buffer_header_mem_lock(self.p.as_ptr()), "could not lock buffer")?;
            let b = self.p.as_ref();
            let rv = f(
                flags,
                std::slice::from_raw_parts(b.data.add(b.offset as usize), b.length as usize)
            );
            ffi::mmal_buffer_header_mem_unlock(self.p.as_ptr());
            rv
        }
    }

    fn new(buffer_ptr: *mut ffi::MMAL_BUFFER_HEADER_T) -> Option<Self> {
        let p = NonNull::new(buffer_ptr)?;
        Some(Self { p })
    }
    
}

impl Clone for BufferRef {
    fn clone(&self) -> Self {
        unsafe { ffi::mmal_buffer_header_acquire(self.p.as_ptr()); }
        Self { p: self.p.clone() }
    }
}

impl Drop for BufferRef {
    fn drop(&mut self) {
        unsafe { ffi::mmal_buffer_header_release(self.p.as_ptr()) }
    }
}

//------------------------------------------------------------------------------------------------------------------------------

/*
pub struct PoolHandle {
    p: NonNull<ffi::MMAL_POOL_T>,
}

impl PoolHandle {
    pub fn create(headers: u32, payload_size: u32) -> Result<Self> {
        let pool_ptr = unsafe { ffi::mmal_pool_create(headers, payload_size) };
        let p = NonNull::new(pool_ptr)
            .ok_or_else(|| MmalError::no_status("Unable to create pool".to_owned()))?;
        Ok(Self { p })
    }

    unsafe fn get_queue(&self) -> *mut ffi::MMAL_QUEUE_T {
        self.p.as_ref().queue
    }

    pub fn get_buffer(&self) -> Option<BufferRef> {
        unsafe {
            let buffer_ptr = ffi::mmal_queue_get(self.p.as_ref().queue);
            let p = NonNull::new(buffer_ptr)?;
            Some(BufferRef { p })
        }
    }
}

impl Drop for PoolHandle {
    fn drop(&mut self) {
        unsafe { ffi::mmal_pool_destroy(self.p.as_ptr()) }
    }
}
*/
//------------------------------------------------------------------------------------------------------------------------------


pub struct PortPoolHandle<P: ComponentPort> {
    c: ComponentHandle<P::E>,
    port: NonNull<ffi::MMAL_PORT_T>,
    pool: NonNull<ffi::MMAL_POOL_T>
}

impl<P: ComponentPort> PortPoolHandle<P> {
    pub fn create(c: ComponentHandle<P::E>) -> Result<Self> {
        unsafe { 
            let port = P::get_port(&c);

            let port = NonNull::new(port)
                .ok_or_else(|| MmalError::with_cause(Cause::GetPort))?;

            let pool = ffi::mmal_port_pool_create(
                port.as_ptr(),
                port.as_ref().buffer_num, 
                port.as_ref().buffer_size
            );
            let pool = NonNull::new(pool)
                .ok_or_else(|| MmalError::with_cause(Cause::CreatePool))?;
            
            Ok(Self { c, port, pool })          
        }
    }

    pub fn get_component(&self) -> &ComponentHandle<P::E> { &self.c }
    pub fn get_port(&self) -> *mut ffi::MMAL_PORT_T { self.port.as_ptr() }

    /// Gets one buffer from the pool's queue and sends it to the port
    pub fn feed_one(&self) -> Result<()> {
        unsafe {
            let buffer_ptr = ffi::mmal_queue_get(self.pool.as_ref().queue);
            if let Some(b) = NonNull::new(buffer_ptr) {
                let status = ffi::mmal_port_send_buffer(self.port.as_ptr(), b.as_ptr());
                cst!(status, "{}: could not send buffer", P::name())
            } else {
                // TODO this should be logged error, not fatal
                Err(MmalError::with_cause(Cause::QueueEmpty))
            }
        }
    }
    /// Drains all available buffers while sending them to the port
    pub fn feed_all(&self) -> Result<()> {
        unsafe {
            while let Some(b) = NonNull::new(ffi::mmal_queue_get(self.pool.as_ref().queue)) {
                let status = ffi::mmal_port_send_buffer(self.port.as_ptr(), b.as_ptr());
                cst!(status, "{}: could not send buffer", P::name())?;
            }
            Ok(())
        }        
    }
    pub fn get_buffer(&self) -> Option<BufferRef> {
        unsafe {
            let buffer_ptr = ffi::mmal_queue_get(self.pool.as_ref().queue);
            let p = NonNull::new(buffer_ptr)?;
            Some(BufferRef { p })
        }
    }
}

impl<P: ComponentPort> Drop for PortPoolHandle<P> {
    fn drop(&mut self) {
        unsafe { ffi::mmal_port_pool_destroy(self.port.as_ptr(), self.pool.as_ptr()) }
    }
}


//------------------------------------------------------------------------------------------------------------------------------

pub struct QueueHandle {
    p: NonNull<ffi::MMAL_QUEUE_T>,
}

impl QueueHandle {
    pub fn create() -> Result<Self> {
        let pool_ptr = unsafe { ffi::mmal_queue_create() };
        let p = NonNull::new(pool_ptr)
            .ok_or_else(|| MmalError::with_cause(Cause::CreateQueue))?;
        Ok(Self { p })
    }

    pub fn get(&self) -> Option<BufferRef> {
        unsafe {
            BufferRef::new(ffi::mmal_queue_get(self.p.as_ptr()))
        }
    }
    pub fn unget(&self, br: BufferRef) {
        unsafe {
            ffi::mmal_queue_put_back(self.p.as_ptr(), br.p.as_ptr());
        }
    }
    pub fn put(&self, b: BufferRef) {
        unsafe {
            ffi::mmal_queue_put(self.p.as_ptr(), b.p.as_ptr())
        }
    }
    pub unsafe fn put_unsafe(&self, b: *mut ffi::MMAL_BUFFER_HEADER_T) {
        ffi::mmal_queue_put(self.p.as_ptr(), b)
    }
    pub fn wait(&self) -> Option<BufferRef> {
        unsafe {
            BufferRef::new(ffi::mmal_queue_wait(self.p.as_ptr()))
        }
    }
    pub fn timedwait(&self, timeout_ms: u32) -> Option<BufferRef> {
        unsafe {
            BufferRef::new(ffi::mmal_queue_timedwait(self.p.as_ptr(), timeout_ms))
        }
    }

}

impl Drop for QueueHandle {
    fn drop(&mut self) {
        unsafe { ffi::mmal_queue_destroy(self.p.as_ptr()) }
    }
}


//------------------------------------------------------------------------------------------------------------------------------

/// This function must be called before any mmal work. Failure to do so will cause errors like:
///
/// mmal: mmal_component_create_core: could not find component 'vc.camera_info'
///
/// See this for more info https://github.com/thaytan/gst-rpicamsrc/issues/28
pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        ffi::bcm_host_init();
        ffi::vcos_init();
        ffi::mmal_vc_init();
    });
}

//------------------------------------------------------------------------------------------------------------------------------

pub trait ParamIO<P: ComponentPort>  {
    fn write(&self, target: &ComponentHandle<P::E>) -> Result<()>;
    fn read(&mut self, target: &ComponentHandle<P::E>) -> Result<()>;
    unsafe fn set_unsafe(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus;
    unsafe fn get_unsafe(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus;
    fn name(&self) -> &'static str;
}


/// Offset of 1st/the only port
pub const DEFAULT_PORT_OFFSET: isize = 0;

/// Port configuration values
pub trait PortConfig {
    unsafe fn apply_format(&self, port: *mut ffi::MMAL_PORT_T);
    unsafe fn apply_buffer_policy(&self, port: *mut ffi::MMAL_PORT_T);
}

/// provides glue between a component type, its port type, and parameter
pub trait ComponentPort {
    type E: ComponentEntity;
    unsafe fn get_port(component: &ComponentHandle<Self::E>) -> *mut ffi::MMAL_PORT_T;
    fn name() -> &'static str;

    fn write<'p>(component: impl AsRef<ComponentHandle<Self::E>>, param: &'p dyn ParamIO<Self>) -> Result<()> 
    where Self: 'p + Sized{
        param.write(component.as_ref())
    }

    fn read<'p>(component: impl AsRef<ComponentHandle<Self::E>>, param: &'p mut dyn ParamIO<Self>) -> Result<()> 
    where Self: 'p + Sized{
        param.read(component.as_ref())
    }

    fn write_multi<'p>(component: impl AsRef<ComponentHandle<Self::E>>, params: impl Iterator<Item=&'p dyn ParamIO<Self>>) -> Result<()> 
    where Self: 'p + Sized{
        unsafe { 
            let port = Self::get_port(component.as_ref());
            for p in params {
                let status = p.set_unsafe(port);
                cst!(status, "unable to set parameter {} on {}", p.name(), Self::name())?;
            }
        }
        Ok(())
    }

    fn read_multi<'p>(component: impl AsRef<ComponentHandle<Self::E>>, params: impl Iterator<Item=&'p mut dyn ParamIO<Self>>) -> Result<()>
    where Self: 'p + Sized {
        unsafe { 
            let port = Self::get_port(component.as_ref());
            for p in params {
                let status = p.get_unsafe(port);
                cst!(status, "unable to get parameter {} on {}", p.name(), Self::name())?;
            }
        }
        Ok(())
    }

    fn configure(component: impl AsRef<ComponentHandle<Self::E>>, config: impl PortConfig) -> Result<()> {
        unsafe {
            let p = Self::get_port(component.as_ref());
            config.apply_format(p);
            let status = ffi::mmal_port_format_commit(p);
            cst!(status, "unable to commit format on {}", Self::name())?;
            config.apply_buffer_policy(p);
            Ok(())
        }
    }

    fn get_buffers_config(component: &ComponentHandle<Self::E>) -> ((u32, u32, u32), (u32, u32, u32)) {
        unsafe {
            let p = & *Self::get_port(component.as_ref());
            (
                (p.buffer_num, p.buffer_num_recommended, p.buffer_num_min),
                (p.buffer_size, p.buffer_size_recommended, p.buffer_size_min)
            )
        }        
    }

    /*
    fn enable(h: &ComponentHandle<Self::E>) -> Result<()> {
        unsafe {
            let port = Self::get_port(h);
            let status = ffi::mmal_port_enable(port, None);
            cst(status, msgf("Unable to enable", Self::name()))?;
        }
        Ok(())
    }
    */

    /*unsafe fn enable_cb(h: &ComponentHandle<Self::E>, cb: unsafe extern "C" fn(*mut ffi::MMAL_PORT_T, *mut ffi::MMAL_BUFFER_HEADER_T)) -> Result<()> {
        let port = Self::get_port(h);
        let status = ffi::mmal_port_enable(port, Some(cb));
        cst(status, msgf("Unable to enable", Self::name()))?;
        Ok(())
    }*/
}



//------------------------------------------------------------------------------------------------------------------------------


pub struct SinkAggregate<P: ComponentPort> {
    q: QueueHandle,
    p: PortPoolHandle<P>,
    w: Option<Waker>,
    _self: NonNull<Self>,
    _p: PhantomPinned
}

impl<P: ComponentPort> SinkAggregate<P> {

    pub fn create(c: ComponentHandle<P::E>) -> Result<Pin<Box<Self>>> {
        let q = QueueHandle::create()?;
        let p = PortPoolHandle::create(c)?;
        let rv = Self { q, p, w: None, _self: NonNull::dangling(), _p: PhantomPinned };
        let mut rv = Box::new(rv);
        rv._self = rv.as_ref().into();
        unsafe { Ok(Pin::new_unchecked(rv)) }
    }
    
    pub fn enable(&self) -> Result<()> {
        let port = self.p.get_port();
        unsafe {
            (*port).userdata = self._self.as_ptr() as *mut ffi::MMAL_PORT_USERDATA_T;
            let status = ffi::mmal_port_enable(port, Some(Self::cb));
            cst!(status, "{}: unable to enable", P::name())?;
        }
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let port = self.p.get_port();
        unsafe {
            let status = ffi::mmal_port_disable(port);
            cst!(status, "{}: unable to disable", P::name())?;
            (*port).userdata = mem::zeroed();
        }
        Ok(())
    }

    unsafe extern "C" fn cb(port: *mut ffi::MMAL_PORT_T, buffer: *mut ffi::MMAL_BUFFER_HEADER_T) {
        let udp = (*port).userdata as *mut Self;
        let ud = if let Some(ud) = udp.as_mut() { ud } else { return };
        ud.q.put_unsafe(buffer);
        ud.w.take().map(|w| w.wake());
    }

    pub fn feed_one(&self) -> Result<()> {
        self.p.feed_one()
    }

    pub fn feed_all(&self) -> Result<()> {
        self.p.feed_all()
    }

    /// Consume a `BufferRef` previously obtained via a method optionally returning `BufferRef` (e.g. `get()`), or via `await`.
    /// 
    /// Note that this is the only way to obtain buffer data.
    /// 
    /// The consumer function shall return `Ok(true)` to indicate that the buffer has been consumed successfully,
    /// `Ok(false)` to indicate that the fuffer shall be returned (ungot) to the queue, or `Err(_)` upon error.
    pub fn consume<R>(&self, b: BufferRef, f: impl FnMut(FrameFlags, &[u8]) -> Result<(bool, R)>) -> Result<(bool, R)>  {
        let (is_consumed, user_data) = b.do_locked(f)?;
        if is_consumed {
            // drop buffer so it's released to the pool
            std::mem::drop(b);
            // send a buffer to the port in place of the consumed
            self.p.feed_one()?;
        } else {
            // return the buffer to the queue
            self.q.unget(b) 
        }
        Ok((is_consumed, user_data))
    }

    /// Wait for a buffer infinitely
    pub fn wait(&self) -> Option<BufferRef> { self.q.wait() }
    /// Wait for a buffer at most specified number of milliseconds
    pub fn timedwait(&self, timeout_ms: u32) -> Option<BufferRef> { self.q.timedwait(timeout_ms) }
    /// Get a buffer from the queue, if any
    pub fn get(&self) -> Option<BufferRef> { self.q.get() }
}




impl<P: ComponentPort> std::future::Future for SinkAggregate<P> {
    type Output = BufferRef;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let sa = self.get_unchecked_mut();

            sa.w = Some(cx.waker().clone());

            if let Some(b) = sa.q.get() {
                Poll::Ready(b)
            } else {
                Poll::Pending
            }
        }
    }
}
