//! # Ruby Wrap Data
//!
//! `ruby_wrap_data` is a crate that provides a fairly low-level means of doing
//! what Ruby's `Data_Wrap_Struct` macro does. That is to say, you can store a
//! pointer to a Rust `Box<T>` inside a Ruby object and get it back out again.
//!
//! Any heap-allocated struct, enum, or whatever should work.
//!
//! ## Example
//!
//! ```rust
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
//!     // call `wrap()`, passing your class and data
//!     ruby_wrap_data::wrap(klass, Some(data))
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
//!     let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
//!     assert!(data.is_some());
//!
//!     // if you try to remove it again, you get None
//!     let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
//!     assert!(data.is_none());
//!
//!     // set new data on the object
//!     let new_data = Box::new(MyValue { val : 2 });
//!     ruby_wrap_data::set(thing, new_data);
//! }
//! ```
//!
//! ## Testing
//!
//! Assuming you're using rbenv (if not, sorry, you're on your own):
//!
//! ```bash
//! CONFIGURE_OPTS=--enable-shared rbenv install
//! RUBY=$(rbenv which ruby) cargo test
//! ```
//!
//! You may need to help Rust find the libruby.so file, like this:
//!
//! ```bash
//! export LD_LIBRARY_PATH=$HOME/.rbenv/versions/2.5.1/lib
//! RUBY=$(rbenv which ruby) cargo test
//! ```
//!
//! ## Copyright and License
//!
//! Copyright Tim Morgan
//!
//! Licensed MIT

extern crate ruby_sys;

use ruby_sys::types::{c_void, CallbackPtr, RBasic, Value};

use std::{mem, ptr};

extern "C" {
    fn rb_define_alloc_func(klass: Value, func: CallbackPtr);
    fn rb_data_object_wrap(
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
/// build your initial data and return the result of calling
/// `wrap(klass, data)`.
///
/// # Arguments
///
/// * `klass` - a Ruby Class
/// * `alloc` - a function taking a Ruby Value and returning a Ruby Value
pub fn define_alloc_func(klass: Value, alloc: fn(Value) -> Value) {
    unsafe { rb_define_alloc_func(klass, alloc as CallbackPtr) };
}

/// Creates a new instance of the given class, wrapping the given
/// heap-allocated data type.
///
/// # Arguments
///
/// * `klass` - a Ruby Class
/// * `data`  - an Option<Box<T>> - the data you wish to embed in the Ruby object or None
pub fn wrap<T>(klass: Value, data: Option<Box<T>>) -> Value {
    let datap = if data.is_some() {
        Box::into_raw(data.unwrap()) as *mut c_void
    } else {
        ptr::null_mut()
    };
    unsafe { rb_data_object_wrap(klass, datap, None, Some(free::<T>)) }
}

/// Removes and returns the wrapped data from the given Ruby object.
/// Returns None if the data is currently NULL.
///
/// # Arguments
///
/// * `object` - a Ruby object
///
/// # Notes
///
/// You will need to specify the data type of the variable since it
/// cannot be inferred:
///
/// ```rust,ignore
/// let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
/// ```
///
/// Also note, if you wish to peek at the data without removing it,
/// you will need to put it back using `set`, like this:
///
/// ```rust,ignore
/// let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
/// // do something
/// ruby_wrap_data::set(thing, data.unwrap());
/// ```
pub fn remove<T>(object: Value) -> Option<Box<T>> {
    let rdata = rdata(object);
    let datap = unsafe { (*rdata).data as *mut T };
    if datap.is_null() {
        None
    } else {
        set_none(object);
        Some(unsafe { Box::from_raw(datap) })
    }
}

/// Sets the wrapped data on the given Ruby object.
///
/// # Arguments
///
/// * `object` - a Ruby object
/// * `data`   - a Box<T> - the data you wish to embed in the Ruby object
pub fn set<T>(object: Value, data: Box<T>) {
    let rdata = rdata(object);
    let datap = Box::into_raw(data) as *mut c_void;
    unsafe { (*rdata).data = datap };
}

extern "C" fn free<T>(data: *mut c_void) {
    // memory is freed when the box goes out of the scope
    let datap = data as *mut T;
    unsafe { Box::from_raw(datap) };
}

fn set_none(object: Value) {
    let rdata = rdata(object);
    unsafe { (*rdata).data = ptr::null_mut() };
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
        vm
    };

    use std::ffi::CString;
    use std::sync::{Once, ONCE_INIT};

    static RUBY_INIT: Once = ONCE_INIT;

    const RB_NIL: Value = Value { value: Nil as usize };

    #[derive(Debug,PartialEq)]
    struct MyValue {
        pub val: u16
    }

    fn alloc(klass: Value) -> Value {
        let data = Box::new(MyValue { val: 1 });
        wrap(klass, Some(data))
    }

    fn alloc_using_none(klass: Value) -> Value {
        wrap::<Option<Box<MyValue>>>(klass, None)
    }

    fn ruby_init() {
        println!("h");
        RUBY_INIT.call_once(|| {
            unsafe { vm::ruby_init() };
            println!("here");
        });
    }

    #[test]
    fn it_works() {
        ruby_init();

        // create our class
        let name = CString::new("Thing").unwrap().into_raw();
        let klass = unsafe { rb_define_class(name, rb_cObject) };

        // set up our alloc function and create the object
        define_alloc_func(klass, alloc);
        let thing = unsafe { rb_class_new_instance(0, &RB_NIL, klass) };

        // the data matches what we put in
        let data: Box<MyValue> = remove(thing).unwrap();
        assert_eq!(*data, MyValue { val: 1 });

        // now it's None
        assert_eq!(remove::<Option<Box<MyValue>>>(thing), None);

        // set new data
        let new_data = Box::new(MyValue { val : 2 });
        set(thing, new_data);

        // looks right
        let data: Box<MyValue> = remove(thing).unwrap();
        assert_eq!(*data, MyValue { val: 2 });

        // create our class
        let name = CString::new("Thing2").unwrap().into_raw();
        let klass = unsafe { rb_define_class(name, rb_cObject) };

        // set up our alloc function and create the object
        define_alloc_func(klass, alloc_using_none);
        let thing = unsafe { rb_class_new_instance(0, &RB_NIL, klass) };

        // the data matches what we put in
        assert!(remove::<Option<Box<MyValue>>>(thing).is_none());
    }
}
