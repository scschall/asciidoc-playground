require 'json'
require 'asciidoctor'

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

# Alle AsciiDoc Dateien finden
adoc_files = Dir.glob('*.adoc')

# Index erstellen
documents = {}
adoc_files.each do |file|
  doc = extract_content(file)
  documents[file] = doc
end

# Lunr Index generieren
lunr_docs = documents.map do |id, doc|
  {
    'id' => id,
    'title' => doc['title'],
    'content' => doc['content']
  }
end

# Index als JavaScript-Datei speichern
File.write('_site/search-index.json', JSON.generate({
  'documents' => documents,
  'index' => `npx lunr-index-build --fields title,content --documents '#{JSON.generate(lunr_docs)}'`
})) 