extern crate libc;
extern crate ruby_sys;

use ruby_sys::{
    class::{ rb_define_class, rb_define_method },
    fixnum::rb_int2inum,
    rb_cObject,
    types::{ CallbackPtr, c_char, c_void, RBasic, Value },
    value::RubySpecialConsts::Nil,
};

use std::mem;

#[allow(dead_code)]
const RB_NIL: Value = Value { value: Nil as usize };

fn c_str_ptr(string: &str) -> *const c_char {
    ::std::ffi::CString::new(string).unwrap().into_raw()
}

pub extern "C" fn free(data: *mut c_void) {
    // Memory is freed when the box goes out of the scope
    unsafe { Box::from_raw(data) };
}

pub extern "C" fn alloc(klass: Value) -> Value {
    let data = Box::new(12);
    let datap = Box::into_raw(data) as *mut c_void;
    unsafe { rb_data_object_wrap(klass, datap, None, Some(free)) }
}

extern "C" {
    pub fn rb_define_alloc_func(klass: Value, func: extern "C" fn(Value) -> Value);
    pub fn rb_data_object_wrap(klass: Value, datap: *mut c_void, mark: Option<extern "C" fn(*mut c_void)>, free: Option<extern "C" fn(*mut c_void)>) -> Value;
}

pub extern "C" fn get_internal_data(itself: Value) -> Value {
    let rdata: *const RData = unsafe { mem::transmute(itself) };
    let the_data = unsafe { Box::from_raw((*rdata).data) };
    unsafe { rb_int2inum(*the_data as isize) }
}

#[repr(C)]
struct RData {
    basic: RBasic,
    dmark: Option<extern "C" fn(*mut c_void)>,
    dfree: Option<extern "C" fn(*mut c_void)>,
    pub data: *mut c_void,
}

#[no_mangle]
pub extern fn init_thing() {
    let name = c_str_ptr("Thing");
    let klass = unsafe { rb_define_class(name, rb_cObject) };
    unsafe { rb_define_alloc_func(klass, alloc) };
    unsafe { rb_define_method(klass, c_str_ptr("get_internal_data"), get_internal_data as CallbackPtr, 0) };
}
