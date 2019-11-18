const path = require('path');
const CopyPlugin = require("copy-webpack-plugin");
const WorkerPlugin = require('worker-plugin');

const testOut = path.resolve(__dirname, 'dist', 'test');
const testIn = path.resolve(__dirname, 'js', 'test');

module.exports = {
  entry: {
    tests: path.join(testIn, 'tests.ts'),
  },
  devtool: 'source-map',
  devServer: {
    contentBase: testOut,
    writeToDisk: true,
  },
  output: {
    libraryTarget: 'umd',
    library: 'wasmMarkerClusterer',
    path: testOut,
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
      path.join(testIn, 'index.html')
    ]),
    new WorkerPlugin(),
  ],
  mode: 'development'
};