include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use std::ffi::CString;

fn to_gostring(c_str: &CString) -> GoString {
    GoString {
        p: c_str.as_ptr(),
        n: c_str.as_bytes().len() as isize,
    }
}

fn start_vm() {
    let project = CString::new("project").unwrap();
    let zone = CString::new("zone").unwrap();
    unsafe { StartVM(to_gostring(&project), to_gostring(&zone)) };
}
