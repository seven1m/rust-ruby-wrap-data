# Ruby Wrap Data

`ruby_wrap_data` is a Rust crate that provides a fairly low-level means of doing
what Ruby's `Data_Wrap_Struct` macro does. That is to say, you can store a Rust
`Box<T>` inside a Ruby object and get it back out again.

Any heap-allocated struct, enum, or whatever should work.

## Example

```
extern crate ruby_sys;
extern crate ruby_wrap_data;

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

struct MyValue {
    pub val: u16
}

fn alloc(klass: Value) -> Value {
    // build your data and put it on the heap
    let data = Box::new(MyValue { val: 1 });
    // call `wrap()`, passing your klass and data
    ruby_wrap_data::wrap(klass, data)
}

fn main() {
    // you may need to start the ruby vm
    unsafe { ruby_init() };

    // create a ruby class and attach your alloc function
    let name = CString::new("Thing").unwrap().into_raw();
    let klass = unsafe { rb_define_class(name, rb_cObject) };
    ruby_wrap_data::define_alloc_func(klass, alloc);

    // create a new instance of the class
    let thing = unsafe { rb_class_new_instance(0, &RB_NIL, klass) };

    // get the data out of the ruby object
    let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
    assert!(data.is_some());

    // if you try to remove it again, you get None
    let data: Option<Box<MyValue>> = ruby_wrap_data::remove(thing);
    assert!(data.is_none());

    // set new data on the object
    let new_data = Box::new(MyValue { val : 2 });
    ruby_wrap_data::set(thing, new_data);
}
```

## Testing

Assuming you're using rbenv (if not, sorry, you're on your own):

```
CONFIGURE_OPTS=--enable-shared rbenv install
RUBY=$(rbenv which ruby) cargo test
```

You may need to help Rust find the libruby.so file, like this:

```
export LD_LIBRARY_PATH=$HOME/.rbenv/versions/2.5.1/lib
RUBY=$(rbenv which ruby) cargo test
```

## Copyright and License

Copyright Tim Morgan

Licensed MIT
