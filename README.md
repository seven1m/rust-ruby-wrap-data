# Ruby Wrap Data

`ruby_wrap_data` is a Rust crate that provides a fairly low-level means of doing
what Ruby's `Data_Wrap_Struct` macro does. That is to say, you can store a Rust
`Box<T>` inside a Ruby object and get it back out again.

Any heap-allocated struct, enum, or whatever should work.

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
