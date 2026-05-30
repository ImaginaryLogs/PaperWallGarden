
for d in wasm-modules/*/; do
if [ -f "$d/Cargo.toml" ]; then
    echo "--------------------------------------------------"
    echo "Automatically compiling module in: $d"
    echo "--------------------------------------------------"
    cd "$d"
    wasm-pack build --target web
    cd ../..
fi
done


cd quartz-plugin-wasm-playground
npm run build
cd ../


echo "Checking if WASM build artifacts exist..."
ls -l wasm-modules/*/pkg/*.wasm || echo "ERROR: No WASM binaries found!"
ls -l wasm-modules/*/pkg/*.js || echo "ERROR: No WASM JS files found!"


mkdir -p static/wasm
cp wasm-modules/*/pkg/*.js static/wasm/
cp wasm-modules/*/pkg/*.wasm static/wasm/

echo "Listing contents of static/wasm/:"
ls -l static/wasm/

mkdir -p .quartz/plugins
if [ ! -f .quartz/plugins/index.ts ]; then
echo 'export const CustomOgImagesEmitterName = "CustomOgImagesEmitter"; export const plugins = {};' > .quartz/plugins/index.ts
fi

# Execute the primary static site generator build
npm run quartz build

# Force mirror your custom compiled assets back into public right before upload
mkdir -p public/wasm
cp -r static/wasm/* public/wasm/


echo "Listing contents of public/wasm:"
ls -R public/wasm || echo "CRITICAL FAILED: Folder public/wasm does not exist!"