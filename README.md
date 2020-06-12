# SDF Designer
Originally this was going to be a way to create CAD models using
signed distance fields, however during technical feasibility I determined
the approach I planned to take was unfeasible.

Nonetheless, the technology is quite cool because it consists of a
programmable shader! Yes, all shaders are programmable, but the
core shader (src/core/src/shaders/simple.frag) reads instructions
and data from a uniform buffer, and uses it to construct the scene on
the fly. In other words the shader is a primitive virtual machine that
operates on some bytecode in the buffer....

My hope was that because there was no warp divergence (all pixels
follow the same path), constructing the SDF would be fast enough.
Unfortunately this was not the case and on integrated GPU's it runs
poorly. I think this may be because of the high number of memory reads,
many jump instructions or instruction cache locality issues, but I have no
real idea and have lost interest in finding out.

So, anyway, don't bother to create virtual machines on a GPU, at least,
don't do it the way I'm doing it here.


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
