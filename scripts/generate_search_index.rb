require 'json'
require 'asciidoctor'
require 'fileutils'

def extract_content(file)
  content = File.read(file)
  doc = Asciidoctor.load(content)
  
  {
    'title' => doc.title,
    'content' => doc.convert.gsub(/<[^>]*>/, ' ').gsub(/\s+/, ' ').strip,
    'url' => File.basename(file, '.*') + '.html',
    'preview' => doc.convert.gsub(/<[^>]*>/, ' ').gsub(/\s+/, ' ').strip[0..200] + '...'
  }
end

# Verzeichnis erstellen
FileUtils.mkdir_p('_site')

# Alle AsciiDoc Dateien finden
adoc_files = Dir.glob('*.adoc')

# Index erstellen
documents = {}
adoc_files.each do |file|
  doc = extract_content(file)
  documents[file] = doc
end

# Suchindex als JSON speichern
File.write('_site/search-index.json', JSON.generate(documents)) 