const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "development",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js",
    library: "WasmMarkerClusterer",
    libraryTarget: 'umd',
    globalObject: 'this',
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      "index.html"
    ]),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      forceMode: "production",
      // outName: "index",
      extraArgs: "--out-name index --target bundler"
    }),
  ]
};