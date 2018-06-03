require 'fiddle'

library = Fiddle::dlopen("target/debug/librust_rbdatatype.so")
init_thing = Fiddle::Function.new(library['init_thing'], [], Fiddle::TYPE_VOID)
init_thing.call

t = Thing.new
p t
p t.get_internal_data
