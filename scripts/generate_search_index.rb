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

# Lunr Index generieren
lunr_docs = documents.map do |id, doc|
  {
    'id' => id,
    'title' => doc['title'],
    'content' => doc['content']
  }
end

# Temporäre JSON-Datei für lunr-index-build erstellen
File.write('_site/temp_docs.json', JSON.generate(lunr_docs))

# Lunr Index mit korrekten Parametern generieren
index = `npx lunr-index-build -r id -f title -f content < _site/temp_docs.json`

# Finale JSON-Datei mit Dokumenten und Index speichern
File.write('_site/search-index.json', JSON.generate({
  'documents' => documents,
  'index' => JSON.parse(index)
}))

# Temporäre Datei löschen
FileUtils.rm('_site/temp_docs.json') 