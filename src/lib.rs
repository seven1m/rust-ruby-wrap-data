extern crate libc;
extern crate ruby_sys;

mod wrap;

use ruby_sys::{class::{rb_define_class, rb_define_method}, rb_cObject, types::{CallbackPtr, Value}};

use std::ffi::CString;
use std::mem;

#[derive(Debug)]
enum MyValue {
    Vec(Vec<MyValue>),
    Str(String),
}

#[no_mangle]
pub extern "C" fn init_thing() {
    let name = CString::new("Thing").unwrap().into_raw();
    let klass = unsafe { rb_define_class(name, rb_cObject) };
    wrap::define_alloc_func(klass, alloc);
    unsafe {
        rb_define_method(
            klass,
            CString::new("show_me_the_data").unwrap().into_raw(),
            show_me_the_data as CallbackPtr,
            0,
        )
    };
    unsafe {
        rb_define_method(
            klass,
            CString::new("mutate_that_data!").unwrap().into_raw(),
            mutate_that_data as CallbackPtr,
            0,
        )
    };
}

fn alloc(klass: Value) -> Value {
    let data: Box<Vec<MyValue>> = Box::new(vec![]);
    wrap::wrap(klass, data)
}

fn show_me_the_data(itself: Value) -> Value {
    let data: Box<Vec<MyValue>> = wrap::get(itself);
    println!("{:?}", data);
    mem::forget(data); // don't free this memory
    itself
}

fn mutate_that_data(itself: Value) -> Value {
    println!("mutating the data");
    let new_data: Box<Vec<MyValue>> = Box::new(vec![
        MyValue::Str("foo".to_string()),
        MyValue::Vec(vec![MyValue::Str("bar".to_string())]),
    ]);
    wrap::set(itself, new_data);
    itself
}
