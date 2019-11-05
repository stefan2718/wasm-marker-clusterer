const path = require('path');
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: {
    index: './js/index.ts',
    tests: './js/tests.ts',
  },
  devtool: 'source-map',
  devServer: {
    contentBase: path.resolve(__dirname, 'dist'),
    writeToDisk: true,
  },
  output: {
    libraryTarget: 'umd',
    library: 'wasmMarkerClusterer',
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    chunkFilename: '[name].js'
  },
  module: {
    rules: [
      {
        test: /\.ts?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [ '.ts', '.js' ],
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "index.html")
    ]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      forceMode: 'production',
      outName: 'webassembly_marker_clusterer'
    }),
  ],
  mode: 'development'
};