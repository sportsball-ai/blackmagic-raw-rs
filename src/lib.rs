#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use] extern crate simple_error;

use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_float};

#[derive(Debug)]
pub struct Error {
    pub result: HRESULT,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "blackmagic raw error: {}", self.result)
    }
}

impl std::error::Error for Error {}

fn void_result(result: HRESULT) -> Result<(), Error> {
    match result {
        0 => Ok(()),
        result => Err(Error{
            result: result,
        }),
    }
}

fn void_option_result(result: HRESULT) -> Result<Option<()>, Error> {
    match result {
        0 => Ok(Some(())),
        1 => Ok(None),
        result => Err(Error{
            result: result,
        }),
    }
}

impl REFIID {
    fn new(b: [u8; 16]) -> REFIID {
        REFIID{
            byte0: b[0],
            byte1: b[1],
            byte2: b[2],
            byte3: b[3],
            byte4: b[4],
            byte5: b[5],
            byte6: b[6],
            byte7: b[7],
            byte8: b[8],
            byte9: b[9],
            byte10: b[10],
            byte11: b[11],
            byte12: b[12],
            byte13: b[13],
            byte14: b[14],
            byte15: b[15],
        }
    }
}

pub struct Factory {
    implementation: *mut IBlackmagicRawFactory,
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

impl Drop for Factory {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl Factory {
    pub fn new_from_path(path: &str) -> Result<Factory, Box<dyn std::error::Error>> {
        unsafe {
            let path = CString::new(path)?;
            let iterator = create_blackmagic_raw_factory_instance_from_path(path.as_ptr());
            if iterator.is_null() {
                bail!("unable to create blackmagic raw factory. the latest drivers may need to be installed");
            }
            return Ok(Factory{
                implementation: iterator,
            });
        }
    }

    pub fn create_codec(&self) -> Result<Codec, Error> {
        let mut codec: *mut IBlackmagicRaw = std::ptr::null_mut();
        unsafe {
            void_result(blackmagic_raw_factory_create_codec(self.implementation, &mut codec))?;
        }
        return Ok(Codec{
            implementation: codec,
        });
    }
}

pub struct Codec {
    implementation: *mut IBlackmagicRaw,
}

unsafe impl Send for Codec {}

impl Drop for Codec {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl Codec {
    pub fn open_clip(&mut self, path: &str) -> Result<Clip, Box<dyn std::error::Error>> {
        let path = CString::new(path)?;
        let mut clip: *mut IBlackmagicRawClip = std::ptr::null_mut();
        unsafe {
            void_result(blackmagic_raw_open_clip(self.implementation, path.as_ptr(), &mut clip))?;
        }
        return Ok(Clip{
            implementation: clip,
        });
    }

    pub fn with_callback<'a, T, F, V>(&mut self, callback: T, f: F) -> Result<V, Box<dyn std::error::Error>>
        where T: Callback + Send + 'a,
              F: FnOnce(&mut Codec) -> V,
    {
        let mut callback: Box<dyn Callback + Send + 'a> = Box::new(callback);
        unsafe {
            self.set_callback(Some(std::mem::transmute::<&mut Box::<dyn Callback + Send + 'a>, &mut Box::<dyn Callback + Send>>(&mut callback)))?;
        }
        let ret = f(self);
        unsafe {
            self.set_callback(None)?;
        }
        Ok(ret)
    }

    /// The caller must ensure that the given callback lives until the callback is unset. Use with_callback for a safer alternative.
    pub unsafe fn set_callback(&mut self, callback: Option<&mut Box<dyn Callback + Send>>) -> Result<(), Error> {
        match callback {
            Some(callback) => {
                let callback = create_blackmagic_raw_callback(callback as *mut Box<dyn Callback + Send> as *mut c_void);
                let result = void_result(blackmagic_raw_set_callback(self.implementation, callback));
                blackmagic_raw_unknown_release(callback as *mut IUnknown);
                result
            },
            None => {
                void_result(blackmagic_raw_set_callback(self.implementation, std::ptr::null_mut()))
            },
        }
    }

    pub fn flush_jobs(&mut self) -> Result<(), Error> {
        unsafe {
            void_result(blackmagic_raw_flush_jobs(self.implementation))
        }
    }
}

pub struct Clip {
    implementation: *mut IBlackmagicRawClip,
}

unsafe impl Send for Clip {}

impl Drop for Clip {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl Clip {
    unsafe fn new_ref(clip: *mut IBlackmagicRawClip) -> Clip {
        blackmagic_raw_unknown_add_ref(clip as *mut IUnknown);
        Clip{
            implementation: clip,
        }
    }

    pub fn get_width(&mut self) -> Result<u32, Error> {
        let mut width = 0;
        unsafe {
            void_result(blackmagic_raw_clip_get_width(self.implementation, &mut width))?;
        }
        return Ok(width)
    }

    pub fn get_height(&mut self) -> Result<u32, Error> {
        let mut height = 0;
        unsafe {
            void_result(blackmagic_raw_clip_get_height(self.implementation, &mut height))?;
        }
        return Ok(height)
    }

    pub fn get_frame_rate(&mut self) -> Result<f32, Error> {
        let mut frame_rate = 0.0;
        unsafe {
            void_result(blackmagic_raw_clip_get_frame_rate(self.implementation, &mut frame_rate))?;
        }
        return Ok(frame_rate)
    }

    pub fn get_frame_count(&mut self) -> Result<u64, Error> {
        let mut frame_count = 0;
        unsafe {
            void_result(blackmagic_raw_clip_get_frame_count(self.implementation, &mut frame_count))?;
        }
        return Ok(frame_count)
    }

    pub fn get_metadata_iterator(&mut self) -> Result<MetadataIterator, Error> {
        let mut iface: *mut IBlackmagicRawMetadataIterator = std::ptr::null_mut();
        unsafe {
            void_result(blackmagic_raw_clip_get_metadata_iterator(self.implementation, &mut iface))?;
        }
        return Ok(MetadataIterator{
            implementation: iface,
        })
    }

    unsafe fn query_interface<T>(&self, iid: REFIID) -> Result<Option<*mut T>, Error> {
            let mut iface: *mut T = std::ptr::null_mut();
            Ok(void_option_result(blackmagic_raw_unknown_query_interface(self.implementation as *mut IUnknown, iid, std::mem::transmute::<&mut *mut T, &mut *mut c_void>(&mut iface)))?.map(|_| iface))
    }

    pub fn get_audio(&mut self) -> Result<Option<ClipAudio>, Error> {
        unsafe {
            Ok(self.query_interface::<IBlackmagicRawClipAudio>(REFIID::new([0x76,0xD4,0xAC,0xED,0xE0,0xD6,0x45,0xBB,0xB5,0x47,0x56,0xB7,0x43,0x5B,0x2A,0x1D]))?.map(|audio| ClipAudio{
                implementation: audio,
            }))
        }
    }

    pub fn create_job_read_frame(&mut self, frame: u64) -> Result<Job, Error> {
        let mut job: *mut IBlackmagicRawJob = std::ptr::null_mut();
        unsafe {
            void_result(blackmagic_raw_clip_create_job_read_frame(self.implementation, frame, &mut job))?;
        }
        return Ok(Job{
            implementation: job,
        });
    }

    // TODO: support attributes arguments
    pub fn create_job_trim(&mut self, file_name: String, frame_index: u64, frame_count: u64) -> Result<Job, Box<dyn std::error::Error>> {
        let mut job: *mut IBlackmagicRawJob = std::ptr::null_mut();
        let file_name = CString::new(file_name)?;
        unsafe {
            void_result(blackmagic_raw_clip_create_job_trim(self.implementation, file_name.as_ptr(), frame_index, frame_count, std::ptr::null_mut(), std::ptr::null_mut(), &mut job))?;
        }
        return Ok(Job{
            implementation: job,
        });
    }
}

pub struct ClipAudio {
    implementation: *mut IBlackmagicRawClipAudio,
}

unsafe impl Send for ClipAudio {}

impl Drop for ClipAudio {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl ClipAudio {
    pub fn get_channel_count(&mut self) -> Result<u32, Error> {
        let mut ret = 0;
        unsafe {
            void_result(blackmagic_raw_clip_audio_get_channel_count(self.implementation, &mut ret))?;
        }
        return Ok(ret)
    }

    pub fn get_sample_rate(&mut self) -> Result<u32, Error> {
        let mut ret = 0;
        unsafe {
            void_result(blackmagic_raw_clip_audio_get_sample_rate(self.implementation, &mut ret))?;
        }
        return Ok(ret)
    }

    pub fn get_sample_count(&mut self) -> Result<u64, Error> {
        let mut ret = 0;
        unsafe {
            void_result(blackmagic_raw_clip_audio_get_sample_count(self.implementation, &mut ret))?;
        }
        return Ok(ret)
    }
}

pub struct Job {
    implementation: *mut IBlackmagicRawJob,
}

unsafe impl Send for Job {}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl Job {
    unsafe fn new_ref(job: *mut IBlackmagicRawJob) -> Job {
        blackmagic_raw_unknown_add_ref(job as *mut IUnknown);
        Job{
            implementation: job,
        }
    }

    pub fn submit(&mut self) -> Result<(), Error> {
        unsafe {
            void_result(blackmagic_raw_job_submit(self.implementation))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceFormat(pub u32);

impl ResourceFormat {
    pub const FORMAT_RGBAU8: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBAU8);
    pub const FORMAT_BGRAU8: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatBGRAU8);
    pub const FORMAT_RGBU16: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBU16);
    pub const FORMAT_RGBAU16: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBAU16);
    pub const FORMAT_BGRAU16: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatBGRAU16);
    pub const FORMAT_RGBU16_PLANAR: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBU16Planar);
    pub const FORMAT_RGBF32: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBF32);
    pub const FORMAT_RGBF32_PLANAR: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatRGBF32Planar);
    pub const FORMAT_BGRAF32: ResourceFormat = ResourceFormat(_BlackmagicRawResourceFormat_blackmagicRawResourceFormatBGRAF32);
}

pub struct Frame {
    implementation: *mut IBlackmagicRawFrame,
}

unsafe impl Send for Frame {}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl Frame {
    unsafe fn new_ref(frame: *mut IBlackmagicRawFrame) -> Frame {
        blackmagic_raw_unknown_add_ref(frame as *mut IUnknown);
        Frame{
            implementation: frame,
        }
    }

    pub fn set_resource_format(&mut self, format: ResourceFormat) -> Result<(), Error> {
        unsafe {
            void_result(blackmagic_raw_frame_set_resource_format(self.implementation, format.0))
        }
    }

    pub fn create_job_decode_and_process_frame(&mut self, clip_processing_attributes: Option<ClipProcessingAttributes>, frame_processing_attributes: Option<FrameProcessingAttributes>) -> Result<Job, Error> {
        let mut job: *mut IBlackmagicRawJob = std::ptr::null_mut();
        unsafe {
            void_result(blackmagic_raw_frame_create_job_decode_and_process_frame(self.implementation, match clip_processing_attributes {
                Some(obj) => obj.implementation,
                None => std::ptr::null_mut(),
            }, match frame_processing_attributes {
                Some(obj) => obj.implementation,
                None => std::ptr::null_mut(),
            }, &mut job))?;
        }
        return Ok(Job{
            implementation: job,
        });
    }
}

pub struct ProcessedImage {
    implementation: *mut IBlackmagicRawProcessedImage,
}

unsafe impl Send for ProcessedImage {}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

impl ProcessedImage {
    unsafe fn new_ref(img: *mut IBlackmagicRawProcessedImage) -> ProcessedImage {
        blackmagic_raw_unknown_add_ref(img as *mut IUnknown);
        ProcessedImage{
            implementation: img,
        }
    }

    pub fn get_width(&mut self) -> Result<u32, Error> {
        let mut out = 0;
        unsafe {
            void_result(blackmagic_raw_processed_image_get_width(self.implementation, &mut out))?
        }
        Ok(out)
    }

    pub fn get_height(&mut self) -> Result<u32, Error> {
        let mut out = 0;
        unsafe {
            void_result(blackmagic_raw_processed_image_get_height(self.implementation, &mut out))?
        }
        Ok(out)
    }

    pub fn get_resource_size_bytes(&mut self) -> Result<u32, Error> {
        let mut out = 0;
        unsafe {
            void_result(blackmagic_raw_processed_image_get_resource_size_bytes(self.implementation, &mut out))?
        }
        Ok(out)
    }

    pub fn get_resource(&mut self) -> Result<&[u8], Error> {
        let len = self.get_resource_size_bytes()?;
        unsafe {
            let mut buf: *mut c_void = std::ptr::null_mut();
            void_result(blackmagic_raw_processed_image_get_resource(self.implementation, &mut buf))?;
            Ok(std::slice::from_raw_parts(buf as *mut u8, len as usize))
        }
    }
}

pub struct ClipProcessingAttributes {
    implementation: *mut IBlackmagicRawClipProcessingAttributes,
}

unsafe impl Send for ClipProcessingAttributes {}

impl Drop for ClipProcessingAttributes {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

pub struct FrameProcessingAttributes {
    implementation: *mut IBlackmagicRawFrameProcessingAttributes,
}

unsafe impl Send for FrameProcessingAttributes {}

impl Drop for FrameProcessingAttributes {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

pub trait Callback {
    fn read_complete(&mut self, _job: Job, _result: Result<Frame, Error>) {}
    fn decode_complete(&mut self, _job: Job, _result: Result<(), Error>) {}
    fn process_complete(&mut self, _job: Job, _result: Result<ProcessedImage, Error>) {}
    fn trim_progress(&mut self, _job: Job, _progress: f32) {}
    fn trim_complete(&mut self, _job: Job, _result: Result<(), Error>) {}
    fn sidecar_metadata_parse_warning(&mut self, _clip: Clip, _filename: &str, _line_number: u32, _info: &str) {}
    fn sidecar_metadata_parse_error(&mut self, _clip: Clip, _filename: &str, _line_number: u32, _info: &str) {}
    fn prepare_pipeline_complete(&mut self, _result: Result<(), Error>) {}
}

#[no_mangle]
unsafe extern "C" fn callback_read_complete(implementation: *mut Box<dyn Callback>, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
    let implementation = &mut *implementation;
    implementation.read_complete(Job::new_ref(job), void_result(result).map(|_| Frame::new_ref(frame)));
}

#[no_mangle]
unsafe extern "C" fn callback_decode_complete(implementation: *mut Box<dyn Callback>, job: *mut IBlackmagicRawJob, result: HRESULT) {
    let implementation = &mut *implementation;
    implementation.decode_complete(Job::new_ref(job), void_result(result));
}

#[no_mangle]
unsafe extern "C" fn callback_process_complete(implementation: *mut Box<dyn Callback>, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
    let implementation = &mut *implementation;
    implementation.process_complete(Job::new_ref(job), void_result(result).map(|_| ProcessedImage::new_ref(processed_image)));
}

#[no_mangle]
unsafe extern "C" fn callback_trim_progress(implementation: *mut Box<dyn Callback>, job: *mut IBlackmagicRawJob, progress: c_float) {
    let implementation = &mut *implementation;
    implementation.trim_progress(Job::new_ref(job), progress as _);
}

#[no_mangle]
unsafe extern "C" fn callback_trim_complete(implementation: *mut Box<dyn Callback>, job: *mut IBlackmagicRawJob, result: HRESULT) {
    let implementation = &mut *implementation;
    implementation.trim_complete(Job::new_ref(job), void_result(result));
}

#[no_mangle]
unsafe extern "C" fn callback_sidecar_metadata_parse_warning(implementation: *mut Box<dyn Callback>, clip: *mut IBlackmagicRawClip, filename: *const c_char, line_number: u32, info: *const c_char) {
    let implementation = &mut *implementation;
    implementation.sidecar_metadata_parse_warning(Clip::new_ref(clip), CStr::from_ptr(filename).to_str().unwrap_or(""), line_number, CStr::from_ptr(info).to_str().unwrap_or(""));
}

#[no_mangle]
unsafe extern "C" fn callback_sidecar_metadata_parse_error(implementation: *mut Box<dyn Callback>, clip: *mut IBlackmagicRawClip, filename: *const c_char, line_number: u32, info: *const c_char) {
    let implementation = &mut *implementation;
    implementation.sidecar_metadata_parse_error(Clip::new_ref(clip), CStr::from_ptr(filename).to_str().unwrap_or(""), line_number, CStr::from_ptr(info).to_str().unwrap_or(""));
}

#[no_mangle]
unsafe extern "C" fn callback_prepare_pipeline_complete(implementation: *mut Box<dyn Callback>, _user_data: *mut c_void, result: HRESULT) {
    let implementation = &mut *implementation;
    // TODO: pass along user data?
    implementation.prepare_pipeline_complete(void_result(result));
}

pub struct MetadataIterator {
    implementation: *mut IBlackmagicRawMetadataIterator,
}

unsafe impl Send for MetadataIterator {}

impl Drop for MetadataIterator {
    fn drop(&mut self) {
        unsafe {
            blackmagic_raw_unknown_release(self.implementation as *mut IUnknown);
        }
    }
}

pub enum Value {
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Float(f32),
    String(String),
    Array(Vec<Value>),
}

impl Value {
    unsafe fn new_from_safe_array(arr: *mut SafeArray) -> Result<Option<Value>, Error> {
        let mut t = _BlackmagicRawVariantType_blackmagicRawVariantTypeEmpty;
        void_result(SafeArrayGetVartype(arr, &mut t))?;

        let mut u = 0;
        void_result(SafeArrayGetUBound(arr, 1, &mut u))?;

        let mut l = 0;
        void_result(SafeArrayGetLBound(arr, 1, &mut l))?;

		let len = (u - l) + 1;

        let mut data: *mut c_void = std::ptr::null_mut();
        void_result(SafeArrayAccessData(arr, &mut data))?;

        let ret = match t {
            _BlackmagicRawVariantType_blackmagicRawVariantTypeU8 => {
                let slice = std::slice::from_raw_parts(data as *mut u8, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::UInt8(*v)).collect()))
            },
            _BlackmagicRawVariantType_blackmagicRawVariantTypeS16 => {
                let slice = std::slice::from_raw_parts(data as *mut i16, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::Int16(*v)).collect()))
            },
            _BlackmagicRawVariantType_blackmagicRawVariantTypeU16 => {
                let slice = std::slice::from_raw_parts(data as *mut u16, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::UInt16(*v)).collect()))
            },
            _BlackmagicRawVariantType_blackmagicRawVariantTypeS32 => {
                let slice = std::slice::from_raw_parts(data as *mut i32, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::Int32(*v)).collect()))
            },
            _BlackmagicRawVariantType_blackmagicRawVariantTypeU32 => {
                let slice = std::slice::from_raw_parts(data as *mut u32, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::UInt32(*v)).collect()))
            },
            _BlackmagicRawVariantType_blackmagicRawVariantTypeFloat32 => {
                let slice = std::slice::from_raw_parts(data as *mut f32, len as _);
                Some(Value::Array(slice.iter().map(|v| Value::Float(*v)).collect()))
            },
            _ => None,
        };

        void_result(SafeArrayUnaccessData(arr))?;

        return Ok(ret)
    }
}

impl std::iter::Iterator for MetadataIterator {
    type Item = (String, Value);

    fn next(&mut self) -> Option<(String, Value)> {
        let mut value = Variant{
            vt: _BlackmagicRawVariantType_blackmagicRawVariantTypeEmpty,
            __bindgen_anon_1: Variant__bindgen_ty_1{
                iVal: 0,
            },
        };

        loop {
            unsafe {
                match void_option_result(blackmagic_raw_metadata_iterator_next(self.implementation)) {
                    Ok(Some(_)) => {},
                    _ => return None,
                };

                let mut buf: *mut Buffer = std::ptr::null_mut();
                let key = match void_result(blackmagic_raw_metadata_iterator_get_key(self.implementation, &mut buf)) {
                    Ok(_) => {
                        let key = std::ffi::CStr::from_ptr(buffer_data(buf) as *const c_char).to_str().unwrap_or("").to_string();
                        buffer_release(buf);
                        key
                    },
                    _ => return None,
                };

                VariantInit(&mut value);

                match void_result(blackmagic_raw_metadata_iterator_get_data(self.implementation, &mut value)) {
                    Ok(()) => {},
                    _ => return None,
                };

                let ret = match value.vt {
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeS16 => Value::Int16(value.__bindgen_anon_1.iVal),
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeU16 => Value::UInt16(value.__bindgen_anon_1.uiVal),
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeS32 => Value::Int32(value.__bindgen_anon_1.intVal),
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeU32 => Value::UInt32(value.__bindgen_anon_1.uintVal),
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeFloat32 => Value::Float(value.__bindgen_anon_1.fltVal),
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeString => {
                        let mut buf: *mut Buffer = std::ptr::null_mut();
                        blackmagic_raw_variant_get_string(&mut value, &mut buf);
                        let s = std::ffi::CStr::from_ptr(buffer_data(buf) as *const c_char).to_str().unwrap_or("").to_string();
                        buffer_release(buf);
                        Value::String(s)
                    },
                    _BlackmagicRawVariantType_blackmagicRawVariantTypeSafeArray => match Value::new_from_safe_array(value.__bindgen_anon_1.parray) {
                        Ok(Some(v)) => v,
                        Ok(None) => {
                            VariantClear(&mut value);
                            continue;
                        }
                        Err(_) => {
                            VariantClear(&mut value);
                            return None;
                        }
                    },
                    _ => {
                        VariantClear(&mut value);
                        continue;
                    }
                };

                VariantClear(&mut value);

                return Some((key, ret));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = Factory::new_from_path("/foo");
    }
}
