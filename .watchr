require 'open3'

def compile
  puts
  puts '========================================='
  puts
  _, stdout, wait_thr = Open3.popen2('cargo test')
  print stdout.getc until stdout.eof?
  wait_thr.value.success?
end

watch('^src/.*|^test\.rb') { compile }
