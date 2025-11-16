use std::{
    ffi::{c_char, c_int, c_uint, c_void},
    marker::PhantomData,
};

/// Raw handle to the pulseaudio simple api object
type PaSimpleRaw = *mut c_void;

#[repr(C)]
struct PaSampleSpecRaw {
    format: c_int,
    rate: c_uint,
    channels: c_char,
}

#[link(name = "pulse-simple")]
unsafe extern "C" {
    fn pa_simple_new(
        server: *const c_char,
        name: *const c_char,
        dir: c_int,
        dev: *const c_char,
        stream_name: *const c_char,
        sample_spec: *const PaSampleSpecRaw,
        chan_map: *const c_void,
        attr: *const c_void,
        err: *mut c_int,
    ) -> PaSimpleRaw;
    fn pa_simple_write(simple: PaSimpleRaw, data: *const c_void, len: usize, err: *mut c_int);
    fn pa_simple_drain(simple: PaSimpleRaw, err: *mut c_int);
    fn pa_simple_free(simple: PaSimpleRaw);
}

#[link(name = "pulse")]
unsafe extern "C" {
    fn pa_strerror(err: c_int) -> *mut c_char;
}

/// Internal convenience function for error strings
fn err_to_string(err: c_int) -> String {
    unsafe {
        std::ffi::CString::from_raw(pa_strerror(err))
            .to_string_lossy()
            .to_string()
    }
}

#[repr(i32)]
enum RawSample {
    U8 = 0,
    S16LE = 3,
    S16BE = 4,
    FLOAT32LE = 5,
    FLOAT32BE = 6,
    S32LE = 7,
    S32BE = 8,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamDirection {
    Playback,
    Capture,
}

impl StreamDirection {
    fn into_c(self) -> c_int {
        match self {
            Self::Capture => 0,
            Self::Playback => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SampleSpec {
    rate: u32,
    n_channels: u8,
}

impl SampleSpec {
    pub fn new(rate: u32, n_channels: u8) -> Self {
        Self { rate, n_channels }
    }
    fn into_c(self) -> PaSampleSpecRaw {
        PaSampleSpecRaw {
            format: -1,
            rate: self.rate,
            channels: self.n_channels as i8,
        }
    }
}

#[derive(Debug)]
pub struct Simple<T: PSimple> {
    raw_handle: PaSimpleRaw,
    _dont: PhantomData<T>,
}

pub trait PSimple: Copy + 'static {
    const FORMAT: c_int;
}

impl PSimple for u8 {
    const FORMAT: c_int = RawSample::U8 as c_int;
}

impl PSimple for i16 {
    #[cfg(target_endian = "little")]
    const FORMAT: c_int = RawSample::S16LE as c_int;
    #[cfg(target_endian = "big")]
    const FORMAT: c_int = RawSample::S16BE as c_int;
}

impl PSimple for f32 {
    #[cfg(target_endian = "little")]
    const FORMAT: c_int = RawSample::FLOAT32LE as c_int;
    #[cfg(target_endian = "big")]
    const FORMAT: c_int = RawSample::FLOAT32BE as c_int;
}

impl PSimple for i32 {
    #[cfg(target_endian = "little")]
    const FORMAT: c_int = RawSample::S32LE as c_int;
    #[cfg(target_endian = "big")]
    const FORMAT: c_int = RawSample::S32BE as c_int;
}

impl<T: PSimple> Simple<T> {
    /// Create a new simple pulseaudio object
    pub fn new(
        stream_name: &str,
        direction: StreamDirection,
        sample_spec: SampleSpec,
    ) -> Result<Self, String> {
        let c_string = std::ffi::CString::new(stream_name).expect("Failed to create CString");

        let mut sam_raw = sample_spec.into_c();
        sam_raw.format = <T as PSimple>::FORMAT as i32;

        let c_char_ptr: *const c_char = c_string.as_ptr();

        let mut err: c_int = 0;

        let handle = unsafe {
            pa_simple_new(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                direction.into_c(),
                std::ptr::null_mut(),
                c_char_ptr,
                &sam_raw,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut err,
            )
        };
        if err != 0 {
            return Err(err_to_string(err));
        }
        Ok(Self {
            raw_handle: handle,
            _dont: PhantomData::default(),
        })
    }

    /// Write `bytes.len()` number of samples to pulse
    pub fn write(&mut self, bytes: &[T]) -> Result<(), String> {
        fn as_bytes<T>(slice: &[T]) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts_mut(
                    slice.as_ptr() as *mut u8,
                    slice.len() * std::mem::size_of::<T>(),
                )
            }
        }
        let bytes = as_bytes(bytes);

        let mut err: c_int = 0;
        unsafe {
            pa_simple_write(
                self.raw_handle,
                bytes as *const [u8] as *const c_void,
                bytes.len(),
                &mut err,
            );
        }
        if err != 0 {
            return Err(err_to_string(err));
        }

        Ok(())
    }

    /// Write a single sample (discouraged?)
    pub fn write_single(&mut self, b: T) -> Result<(), String> {
        self.write(&[b])
    }

    /// Drain (wait until all data has been processed by pulseaudio) the pulseaudio simple api
    /// object
    pub fn drain(&mut self) -> Result<(), String> {
        let mut err: c_int = 0;
        unsafe {
            pa_simple_drain(self.raw_handle, &mut err);
        }
        if err != 0 {
            return Err(err_to_string(err));
        }
        Ok(())
    }
}

impl<T: PSimple> std::ops::Drop for Simple<T> {
    fn drop(&mut self) {
        unsafe {
            pa_simple_drain(self.raw_handle, std::ptr::null_mut());
            pa_simple_free(self.raw_handle);
        }
    }
}
