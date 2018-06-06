extern crate libc;
extern crate ruby_sys;

#[derive(Debug)]
enum MyValue {
    Vec(Box<Vec<MyValue>>),
    Str(String),
}

use ruby_sys::{array::{rb_ary_entry, rb_ary_len, rb_ary_new, rb_ary_push},
               class::{rb_define_class, rb_define_method}, rb_cObject,
               string::{rb_str_new_cstr, rb_string_value_cstr},
               types::{c_char, c_void, CallbackPtr, RBasic, Value},
               value::{RubySpecialConsts::Nil, ValueType}};

use std::mem;
use std::ffi::{CStr, CString};

#[repr(C)]
struct RData {
    basic: RBasic,
    dmark: Option<extern "C" fn(*mut c_void)>,
    dfree: Option<extern "C" fn(*mut c_void)>,
    pub data: *mut c_void,
}

extern "C" {
    pub fn rb_define_alloc_func(klass: Value, func: extern "C" fn(Value) -> Value);
    pub fn rb_data_object_wrap(
        klass: Value,
        datap: *mut c_void,
        mark: Option<extern "C" fn(*mut c_void)>,
        free: Option<extern "C" fn(*mut c_void)>,
    ) -> Value;
}

#[allow(dead_code)]
const RB_NIL: Value = Value {
    value: Nil as usize,
};

fn c_str(string: &str) -> CString {
    CString::new(string).unwrap()
}

fn c_str_ptr(string: &str) -> *const c_char {
    c_str(string).into_raw()
}

pub extern "C" fn free(data: *mut c_void) {
    // memory is freed when the box goes out of the scope
    let datap = data as *mut Vec<MyValue>;
    unsafe { Box::from_raw(datap) };
}

pub extern "C" fn alloc(klass: Value) -> Value {
    let data: Box<Vec<MyValue>> = Box::new(vec![]);
    let datap = Box::into_raw(data) as *mut c_void;
    unsafe { rb_data_object_wrap(klass, datap, None, Some(free)) }
}

pub extern "C" fn get_internal_data(itself: Value) -> Value {
    let rdata: *const RData = unsafe { mem::transmute(itself) };
    let datap = unsafe { (*rdata).data as *mut Vec<MyValue> };
    let the_data = unsafe { Box::from_raw(datap) };
    let result = rb_array_from_internal_values(&the_data);
    mem::forget(the_data); // don't free this memory
    result
}

fn rb_array_from_internal_values(data: &Box<Vec<MyValue>>) -> Value {
    let mut ary = unsafe { rb_ary_new() };
    for elm in data.iter() {
        let item = match elm {
            &MyValue::Vec(ref v) => rb_array_from_internal_values(v),
            &MyValue::Str(ref s) => unsafe { rb_str_new_cstr(c_str_ptr(&s.clone())) },
        };
        ary = unsafe { rb_ary_push(ary, item) }
    }
    ary
}

pub extern "C" fn set_internal_data(itself: Value, data: Value) -> Value {
    let arr = internal_values_from_rb_array(data);
    let datap = Box::into_raw(Box::new(arr)) as *mut c_void;
    let rdata: *mut RData = unsafe { mem::transmute(itself) };
    unsafe { (*rdata).data = datap };
    data
}

fn internal_values_from_rb_array(data: Value) -> Vec<MyValue> {
    let len = unsafe { rb_ary_len(data) };
    let mut arr: Vec<MyValue> = vec![];
    for i in 0..len {
        let item = unsafe { rb_ary_entry(data, i) };
        let item = match item.ty() {
            ValueType::RString => {
                let cstr = unsafe { rb_string_value_cstr(&item) };
                let string = unsafe { CStr::from_ptr(cstr) }
                    .to_string_lossy()
                    .into_owned();
                MyValue::Str(string)
            }
            ValueType::Array => MyValue::Vec(Box::new(internal_values_from_rb_array(item))),
            _ => panic!(),
        };
        arr.push(item);
    }
    arr
}

#[no_mangle]
pub extern "C" fn init_thing() {
    let name = c_str_ptr("Thing");
    let klass = unsafe { rb_define_class(name, rb_cObject) };
    unsafe { rb_define_alloc_func(klass, alloc) };
    unsafe {
        rb_define_method(
            klass,
            c_str_ptr("get_internal_data"),
            get_internal_data as CallbackPtr,
            0,
        )
    };
    unsafe {
        rb_define_method(
            klass,
            c_str_ptr("set_internal_data"),
            set_internal_data as CallbackPtr,
            1,
        )
    };
}
