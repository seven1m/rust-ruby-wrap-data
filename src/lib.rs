//! # Ruby Wrap Data
//!
//! `ruby_wrap_data` is a crate that provides a fairly low-level means of doing
//! what Ruby's `Data_Wrap_Struct` macro does. That is to say, you can store a
//! Rust `Box<T>` inside a Ruby object and get it back out again.
//!
//! Any heap-allocated struct, enum, or whatever should work.
//!
//! ## Example
//!
//! ```
//! extern crate ruby_sys;
//! extern crate ruby_wrap_data;
//!
//! use ruby_sys::{
//!     class::{rb_class_new_instance, rb_define_class},
//!     rb_cObject,
//!     types::Value,
//!     value::RubySpecialConsts::{Nil},
//!     vm::ruby_init
//! };
//!
//! use std::ffi::CString;
//! use std::mem;
//!
//! const RB_NIL: Value = Value { value: Nil as usize };
//!
//! struct MyValue {
//!     pub val: u16
//! }
//!
//! fn alloc(klass: Value) -> Value {
//!     // build your data and put it on the heap
//!     let data = Box::new(MyValue { val: 1 });
//!     // call `wrap()`, passing your klass and data
//!     ruby_wrap_data::wrap(klass, data)
//! }
//!
//! fn main() {
//!     // you may need to start the ruby vm
//!     unsafe { ruby_init() };
//!
//!     // create a ruby class and attach your alloc function
//!     let name = CString::new("Thing").unwrap().into_raw();
//!     let klass = unsafe { rb_define_class(name, rb_cObject) };
//!     ruby_wrap_data::define_alloc_func(klass, alloc);
//!
//!     // create a new instance of the class
//!     let thing = unsafe { rb_class_new_instance(0, &RB_NIL, klass) };
//!
//!     // get the data out of the ruby object
//!     let data: Box<MyValue> = ruby_wrap_data::get(thing);
//!     // forget the data so it's not freed
//!     mem::forget(data);
//!
//!     // set new data on the object
//!     let new_data = Box::new(MyValue { val : 2 });
//!     ruby_wrap_data::set(thing, new_data);
//! }
//! ```
extern crate ruby_sys;

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

/// Defines an 'alloc' function for a Ruby class. Such a function should
/// build your initial data and call `wrap(klass, data)`.
///
/// # Arguments
///
/// * `klass` - a Ruby Class
/// * `alloc` - a function taking a Ruby Value and returning a Ruby Value
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

#[cfg(test)]
mod tests {
    use super::*;

    use ruby_sys::{
        class::{rb_class_new_instance, rb_define_class},
        rb_cObject,
        types::Value,
        value::RubySpecialConsts::{Nil},
        vm::ruby_init
    };

    use std::ffi::CString;
    use std::mem;

    const RB_NIL: Value = Value { value: Nil as usize };

    #[derive(Debug,PartialEq)]
    struct MyValue {
        pub val: u16
    }

    fn alloc(klass: Value) -> Value {
        let data = Box::new(MyValue { val: 1 });
        wrap(klass, data)
    }

    #[test]
    fn it_works() {
        unsafe { ruby_init() };
        let name = CString::new("Thing").unwrap().into_raw();
        let klass = unsafe { rb_define_class(name, rb_cObject) };
        define_alloc_func(klass, alloc);
        let thing = unsafe { rb_class_new_instance(0, &RB_NIL, klass) };
        let data: Box<MyValue> = get(thing);
        assert_eq!(*data, MyValue { val: 1 });
        mem::forget(data);
        let new_data = Box::new(MyValue { val : 2 });
        set(thing, new_data);
        let data: Box<MyValue> = get(thing);
        assert_eq!(*data, MyValue { val: 2 });
        mem::forget(data);
    }
}
