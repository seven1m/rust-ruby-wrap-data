require 'fiddle'

library = Fiddle::dlopen("target/debug/librust_rbdatatype.so")
init_thing = Fiddle::Function.new(library['init_thing'], [], Fiddle::TYPE_VOID)
init_thing.call

thing = Thing.new
p thing
thing.show_me_the_data
thing.mutate_that_data!
thing.show_me_the_data
