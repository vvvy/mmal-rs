use std::marker::PhantomData;
use ffi::MMAL_STATUS_T;
use super::*;


/// Inner parametr type wraps raw MMAL ffi type, e.g. Rational, U32, etc.
pub trait InnerParamType {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus;
    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus;
    fn name() -> &'static str;
}

pub trait Apply<T> {
    fn apply(&mut self, source: T);
}

/// A glue between an InnerParamType and a ComponentPort 
pub struct Param<P, I> {
    i: I,
    p: PhantomData<P>
}

impl<P, I> Param<P, I> {
    pub fn new(value: I) -> Self { Self { i: value, p: PhantomData, } }
    pub fn from_inner<T: Into<I>>(value: T) -> Self { Self::new(value.into()) }
    pub fn from<T>(value: T) -> Self where I: Default+Apply<T> { 
        let mut i = I::default();
        i.apply(value);
        Self::new(i)
    }
    pub fn inner(&self) -> &I { &self.i }
    pub fn inner_mut(&mut self) -> &mut I { &mut self.i }
    pub fn get<'s, T>(&'s self) -> T where T: From<&'s I> { (&self.i).into() }
    pub fn try_get<'s, T>(&'s self) -> Result<T> where T: TryFrom<&'s I, Error = MmalError> { (&self.i).try_into() }
    pub fn set_inner<T>(&mut self, t: T) where T: Into<I> { self.i = t.into() }
    pub fn set<T>(&mut self, source: T) where I: Apply<T> { self.i.apply(source) }
    pub fn into<T: From<I>>(self) -> I { self.i.into() }
}


impl<P, I> AsRef<I> for Param<P, I> {
    fn as_ref(&self) -> &I { &self.i }
}

impl<P, I> AsMut<I> for Param<P, I> {
    fn as_mut(&mut self) -> &mut I { &mut self.i }
}

impl<P, I> Default for Param<P, I> where 
    I: InnerParamType + Default
{
    fn default() -> Self { Self::new(I::default()) }
}

impl<P, I> ParamIO<P> for Param<P, I> where 
    I: InnerParamType, 
    P: ComponentPort {

    fn write(&self, target: &ComponentHandle<P::E>) -> Result<()> {
        let status = unsafe {
            let port = P::get_port(target);
            self.i.set_param(port)
        };
        cst!(status, "Unable to set parameter {} on {}", I::name(), P::name())
    }

    fn read(&mut self, target: &ComponentHandle<P::E>) -> Result<()> {
        let status = unsafe {
            let port = P::get_port(target);
            self.i.get_param(port)  
        };
        cst!(status, "Unable to get parameter {} on {}", I::name(), P::name())
    }

    unsafe fn set_unsafe(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        self.i.set_param(port)
    }
    
    unsafe fn get_unsafe(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        self.i.get_param(port)
    }

    fn name(&self) -> &'static str { I::name() }
}

//------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! param_iter {
    [$($e:expr),+] => { [$($e as &dyn ParamIO<_>),+].into_iter() };
}

#[macro_export]
macro_rules! param_iter_mut {
    [$($e:expr),+] => { [$($e as &mut dyn ParamIO<_>),+].into_iter() };
}

/// Parameter identification
pub trait ParId {
    /// Name of the parameter
    fn name() -> &'static str;
    /// Numeric identifier used in MMAL
    fn n() -> u32;
}

/// Creates a ParId based on MMAL parameter constant
#[macro_export]
macro_rules! idp {
    ($id:ident) => {
        #[allow(non_camel_case_types)]
        pub struct $id;
        impl ParId for $id {
            fn name() -> &'static str { stringify!($id) }
            fn n() -> u32 { ffi::$id }
        }
        
    };
}

/// Injects a Rust enum for a MMAL enumerated parameter 
#[macro_export]
macro_rules! enumize {
    ($enumid:ident, $($int:ident => $ext:ident),+) => {
        #[allow(non_camel_case_types)]
        #[repr(u32)]
        pub enum $enumid { $($int = ffi::$ext),+ }
        impl TryFrom<u32> for $enumid {
            type Error = crate::error::MmalError;
            fn try_from(value: u32) -> Result<Self> {
                match value {
                    $(ffi::$ext => Ok($enumid::$int),)+
                    w => Err(MmalError::new(Cause::InvalidEnumValue, format!("invalid binary value {} for `{}`", w, stringify!($enumid))))
                }
            }
        }
        impl std::str::FromStr for $enumid {
            type Err = MmalError;

            // Required method
            fn from_str(s: &str) -> Result<Self> {
                match s {
                    $(stringify!($int) => Ok($enumid::$int),)+
                    w => Err(MmalError::new(Cause::InvalidEnumValue, format!("invalid string value `{}` for `{}`", w, stringify!($enumid))))
                }
            }
        }
    };
}

/// Injects an inner type for a MMAL enumerated parameter
#[macro_export]
macro_rules! enumerated_inner_type {
    ($typeid:ident, $enumid:ident, $ffitype:ident, $ffitypeid:ident) => {
        enumerated_inner_type!{$typeid, $enumid, $ffitype, $ffitypeid, value}
    };

    ($typeid:ident, $enumid:ident, $ffitype:ident, $ffitypeid:ident, $valuefield:ident) => {
        pub struct $typeid {
            inner: ffi::$ffitype
        }
        
        impl Apply<$enumid> for $typeid {
            fn apply(&mut self, source: $enumid) {
                self.inner.$valuefield = source as u32
            }
        }

        impl TryFrom<&$typeid> for $enumid {
            type Error = MmalError;
            fn try_from(a: &$typeid) -> Result<$enumid> {
                a.inner.$valuefield.try_into()
            }
        }

        impl Default for $typeid{
            fn default() -> Self { 
                let mut cfg: ffi::$ffitype = unsafe { mem::zeroed() };
                cfg.hdr.id = ffi::$ffitypeid as u32;
                cfg.hdr.size = mem::size_of::<ffi::$ffitype>() as u32;
                Self { inner: cfg }
            }
        }
        
        impl InnerParamType for $typeid {
            unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
                ffi::mmal_port_parameter_get(port, &mut self.inner.hdr)
            }
        
            unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
                ffi::mmal_port_parameter_set(port, &self.inner.hdr)
            }
        
            fn name() -> &'static str { stringify!($enumid) }
        }
    };
}

///Initializes a empty parameter struct
#[macro_export]
macro_rules! mmal_param_init {
    ($ty:ident, $id:ident) => { {
        let mut cfg: ffi::$ty = unsafe { mem::zeroed() };
        cfg.hdr.id = ffi::$id as u32;
        cfg.hdr.size = mem::size_of::<ffi::$ty>() as u32;
        cfg
    } };
}

///Implements inner param
#[macro_export]
macro_rules! impl_inner_param_type {
    ($ty:ident) => {
        impl InnerParamType for $ty {
            unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
                ffi::mmal_port_parameter_get(port, &mut self.inner.hdr)
            }
        
            unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
                ffi::mmal_port_parameter_set(port, &self.inner.hdr)
            }
        
            fn name() -> &'static str { stringify!($ty) }
        }
    };
}


///Implements Default for inner param
#[macro_export]
macro_rules! impl_inner_param_default {
    ($ty:ident, $ffi_ty:ident, $ffi_id:ident) => {
        impl Default for $ty {
            fn default() -> Self { 
                Self { inner: mmal_param_init!($ffi_ty, $ffi_id) }
            }
        }
    }
}


//------------------------------------------------------------------------------------------------------------------------------

pub struct Rational<IDP> { inner: ffi::MMAL_RATIONAL_T, _d: PhantomData<IDP> }

impl<IDP> Rational<IDP> {
    pub fn new(num: i32, den: i32) -> Self {
        Self { inner: ffi::MMAL_RATIONAL_T { num, den }, _d: PhantomData }
    }
    pub fn new_scale100(num: i32) -> Self {
        Self::new(num, 100)
    }
    pub fn get_num(&self) -> i32 { self.inner.num }
    pub fn get_den(&self) -> i32 { self.inner.den }
    pub fn set_num(&mut self, num: i32) { self.inner.num = num }
    pub fn set_den(&mut self, den: i32) { self.inner.den = den }
    pub fn get(&self) -> (i32, i32) { (self.inner.num, self.inner.den) }
    pub fn set(&mut self, (num, den): (i32, i32)) {
        self.inner.num = num;
        self.inner.den = den;
    }

    pub fn get_scale100(&self) -> i32 {
        fn rconv(num: i32, den: i32) -> i32 { if den == 100 { num } else { ((num as f32) / (den as f32) * 100.) as i32 } }
        rconv(self.inner.num, self.inner.den)
    }
    pub fn set_scale100(&mut self, num: i32) {
        self.set((num, 100))
    }
}

impl<IDP> Default for Rational<IDP> {
    fn default() -> Self { Self::new(0, 1) }
}

impl<IDP: ParId> InnerParamType for Rational<IDP> {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_get_rational(port, IDP::n(), &mut self.inner)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set_rational(port, IDP::n(), self.inner)
    }

    fn name() -> &'static str { IDP::name() }
}

impl<IDP> Apply<(i32, i32)> for Rational<IDP> {
    fn apply(&mut self, source: (i32, i32)) {
        self.set(source)
    }
}


pub struct Uint32<IDP> { inner: u32, _d: PhantomData<IDP> }

impl<IDP> Uint32<IDP> {
    pub fn new(value: u32) -> Self { Self { inner: value, _d: PhantomData } }
    pub fn get(&self) -> u32 { self.inner }
    pub fn set(&mut self, value: u32) { self.inner = value }
    pub fn inner(&self) -> &u32 { &self.inner }
    pub fn inner_mut(&mut self) -> &mut u32 { &mut self.inner }
}

impl<IDP: ParId> InnerParamType for Uint32<IDP> {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_get_uint32(port, IDP::n(), &mut self.inner)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set_uint32(port, IDP::n(), self.inner)
    }

    fn name() -> &'static str { IDP::name() }
}

impl<IDP> From<&u32> for Uint32<IDP> {
    fn from(value: &u32) -> Self { Self::new(*value) }
}

impl<IDP> From<u32> for Uint32<IDP> {
    fn from(value: u32) -> Self { Self::new(value) }
}

impl<IDP> From<&Uint32<IDP>> for u32 {
    fn from(value: &Uint32<IDP>) -> Self { value.inner }
}

impl<IDP> Default for Uint32<IDP> {
    fn default() -> Self {
        Self { inner: Default::default(), _d: Default::default() }
    }
}

impl<IDP> Apply<u32> for Uint32<IDP> {
    fn apply(&mut self, source: u32) {
        self.set(source)
    }
}



pub struct Int32<IDP> { inner: i32, _d: PhantomData<IDP> }

impl<IDP> Int32<IDP> {
    pub fn new(value: i32) -> Self { Self { inner: value, _d: PhantomData } }
    pub fn get(&self) -> i32 { self.inner }
    pub fn set(&mut self, value: i32) { self.inner = value }
    pub fn inner(&self) -> &i32 { &self.inner }
    pub fn inner_mut(&mut self) -> &mut i32 { &mut self.inner }
}

impl<IDP: ParId> InnerParamType for Int32<IDP> {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_get_int32(port, IDP::n(), &mut self.inner)
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set_int32(port, IDP::n(), self.inner)
    }

    fn name() -> &'static str { IDP::name() }
}

impl<IDP> From<&i32> for Int32<IDP> {
    fn from(value: &i32) -> Self { Self::new(*value) }
}

impl<IDP: ParId> From<i32> for Int32<IDP> {
    fn from(value: i32) -> Self { Self::new(value) }
}

impl<IDP: ParId> From<&Int32<IDP>> for i32 {
    fn from(value: &Int32<IDP>) -> Self { value.inner }
}

impl<IDP> Default for Int32<IDP> {
    fn default() -> Self {
        Self { inner: Default::default(), _d: Default::default() }
    }
}

impl<IDP> Apply<i32> for Int32<IDP> {
    fn apply(&mut self, source: i32) {
        self.set(source)
    }
}


pub struct Boolean<IDP> { inner: bool, _d: PhantomData<IDP> }

impl<IDP> Boolean<IDP> {
    pub fn new(value: bool) -> Self { Self { inner: value, _d: PhantomData } }
    pub fn get(&self) -> bool { self.inner }
    pub fn set(&mut self, value: bool) { self.inner = value }
    pub fn inner(&self) -> &bool { &self.inner }
    pub fn inner_mut(&mut self) -> &mut bool { &mut self.inner }
}

impl<IDP: ParId> InnerParamType for Boolean<IDP> {
    unsafe fn get_param(&mut self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        let mut w = 0i32;
        let status = ffi::mmal_port_parameter_get_boolean(port, IDP::n(), &mut w);
        if status == MMAL_STATUS_T::MMAL_SUCCESS {
            self.inner = w != 0;
        }
        status
    }

    unsafe fn set_param(&self, port: *mut ffi::MMAL_PORT_T) -> MmalStatus {
        ffi::mmal_port_parameter_set_boolean(port, IDP::n(), 
            if self.inner { 1 } else { 0 } as i32)
    }

    fn name() -> &'static str { IDP::name() }
}

impl<IDP> From<&bool> for Boolean<IDP> {
    fn from(value: &bool) -> Self { Self::new(*value) }
}

impl<IDP> From<bool> for Boolean<IDP> {
    fn from(value: bool) -> Self { Self::new(value) }
}

impl<IDP> From<&Boolean<IDP>> for bool {
    fn from(value: &Boolean<IDP>) -> Self { value.inner }
}

impl<IDP> Default for Boolean<IDP> {
    fn default() -> Self {
        Self { inner: Default::default(), _d: Default::default() }
    }
}

impl<IDP> Apply<bool> for Boolean<IDP> {
    fn apply(&mut self, source: bool) {
        self.set(source)
    }
}
