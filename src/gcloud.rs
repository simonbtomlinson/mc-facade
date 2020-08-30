use std::ffi::{CStr, CString};
use std::{convert::TryFrom, os::raw::c_char, marker::PhantomData};

use crate::error::Error;
extern crate libc;

extern "C" {
    fn DoubleString(s: GoString) -> *mut c_char;
}

#[repr(C)]
struct GoString {
    s: *const c_char,
    len: i64,
}

pub fn double() -> String {
    let input = "test string";
    let c_path = CString::new(input).expect("cstring::new failure");
    let go_string = GoString { s: c_path.as_ptr(), len: c_path.as_bytes().len() as i64 };
    let result = unsafe { DoubleString(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str.to_str().expect("It should have worked").to_owned();
    // 
    // unsafe { std::mem::drop(Box::from_raw(result)) }
    string
}