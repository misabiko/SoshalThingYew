import os, json

var j = %* {}
for path in walkFiles("dist/.stage/index-*"):
  let (_, filename, ext) = splitFile path
  j[ext] = %* (filename & ext)

echo "Generated Files: " & $ j
writeFile("generated_files.json", $ j)
