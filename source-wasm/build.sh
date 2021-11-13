wasm-pack build --target web
rm -rf web/pkg
mv pkg web

rm -rf web/assets
cp -r assets web
python3 -m http.server 8089 -d web