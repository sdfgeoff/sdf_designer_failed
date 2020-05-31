build:
	cd src/core; wasm-pack build
	webpack --config webpack.config.js
