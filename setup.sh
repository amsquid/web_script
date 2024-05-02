rm -rf build
cargo build --release
mkdir build
cp target/debug/web_script build/web_script
cp scripts build/scripts -r
cp www build/www -r
cp defaults build/defaults -r