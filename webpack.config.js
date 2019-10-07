const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./js/index.ts"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  module: {
    rules: [
      {
        test: /\.(ts|tsx)?$/,
        loader: 'ts-loader',
        exclude: /node_modules/
      }
    ]
  },
  resolve: {
    extensions: [
      '.tsx',
      '.ts',
      '.js'
    ]
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __dirname,
      forceMode: "production",
      // outName: "index",
      extraArgs: "--out-name index --target bundler"
    }),
  ]
};