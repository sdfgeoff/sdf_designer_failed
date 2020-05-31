# SDF Designer
A browser based CAD program that uses SDF's to do non-destructive
editing.


# Developer notes:
This program has it's core written in rust compiled to webassembly
This is because it contains a significant amount of computation.
My aim is to keep most of it HTML/rust and have as little JS as
possible.

# Building:

1. Install npm (eg `pacman -S npm)
2. Install rust with the wasm toolchain
3. Install wasm-pack
4. Build with "npm run-script build" or "make"
