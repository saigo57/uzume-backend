licenses_dir = './tmp-licenses'

def isFile?(path)
  if FileTest.directory?(path)
    return false
  elsif FileTest.file?(path)
    return true
  else
    raise 'ファイルでもディレクトリでもない'
  end
end

File.open('./NOTICE', 'w') do |f|
  f.puts 'This project includes the following projects.'
  f.puts "\n"

  Dir.glob('**/*', File::FNM_DOTMATCH, base: licenses_dir).each do |file|
    file_path = File.join(licenses_dir, file)
    next unless isFile?(file_path)
    next if file_path.include?('uzume_backend')
  
    license_str = File.read(file_path)
  
    f.puts file
    f.puts '=' * 70
    f.puts license_str
    f.puts '=' * 70
    f.puts "\n"
  end  
end
