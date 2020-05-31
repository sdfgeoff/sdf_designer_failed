const path = require('path');

module.exports = {
  entry: './src/index.js',
  output: {
    filename: 'main.js',
    webassemblyModuleFilename: "sdf_designer.wasm",
    path: path.resolve(__dirname, 'dist'),
  },
  module: {
		rules: [
			{
				test: /\.wasm$/,
				type: "webassembly/experimental"
			}
		]
	},
	mode: "production"
};
