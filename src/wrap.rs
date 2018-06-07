use ruby_sys::types::{c_void, CallbackPtr, RBasic, Value};

use std::mem;

extern "C" {
    pub fn rb_define_alloc_func(klass: Value, func: CallbackPtr);
    pub fn rb_data_object_wrap(
        klass: Value,
        datap: *mut c_void,
        mark: Option<extern "C" fn(*mut c_void)>,
        free: Option<extern "C" fn(*mut c_void)>,
    ) -> Value;
}

#[repr(C)]
struct RData {
    basic: RBasic,
    dmark: Option<extern "C" fn(*mut c_void)>,
    dfree: Option<extern "C" fn(*mut c_void)>,
    pub data: *mut c_void,
}

pub fn define_alloc_func(klass: Value, alloc: fn(Value) -> Value) {
    unsafe { rb_define_alloc_func(klass, alloc as CallbackPtr) };
}

pub fn wrap<T>(klass: Value, data: Box<T>) -> Value {
    let datap = Box::into_raw(data) as *mut c_void;
    unsafe { rb_data_object_wrap(klass, datap, None, Some(free::<T>)) }
}

extern "C" fn free<T>(data: *mut c_void) {
    // memory is freed when the box goes out of the scope
    let datap = data as *mut T;
    unsafe { Box::from_raw(datap) };
}

pub fn get<T>(itself: Value) -> Box<T> {
    let rdata = rdata(itself);
    let datap = unsafe { (*rdata).data as *mut T };
    unsafe { Box::from_raw(datap) }
}

pub fn set<T>(itself: Value, data: Box<T>) {
    let rdata = rdata(itself);
    let datap = Box::into_raw(data) as *mut c_void;
    unsafe { (*rdata).data = datap };
}

fn rdata(object: Value) -> *mut RData {
    unsafe { mem::transmute(object) }
}
