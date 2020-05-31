build:
	cd src/core; wasm-pack build --release
	webpack --config webpack.config.js
