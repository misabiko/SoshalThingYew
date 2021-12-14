import os, json

var j = %* {}
for path in walkFiles("dist/index-*"):
  let (_, filename, ext) = splitFile path
  j[ext] = %* (filename & ext)

echo "Generated Files: " & $ j
writeFile("dist/generated_files.json", $ j)
