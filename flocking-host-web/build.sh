mkdir -p build
clang --target=wasm32 --no-standard-libraries -Wl,--export-all -Wl,--no-entry -I../libflock.include -o build/libflock.wasm ../libflock/src/libflock.c
