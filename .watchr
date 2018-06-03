require 'open3'

def compile
  puts
  puts '========================================='
  puts
  _, stdout, wait_thr = Open3.popen2('cargo build')
  print stdout.getc until stdout.eof?
  wait_thr.value.success?
end

def test
  puts
  puts '========================================='
  puts
  _, stdout, wait_thr = Open3.popen2('ruby test.rb')
  print stdout.getc until stdout.eof?
  wait_thr.value.success?
end

watch('^src/.*|^test\.rb') { compile && test }
