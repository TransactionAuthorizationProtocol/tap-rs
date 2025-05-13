const path = require('path');
const CopyPlugin = require('copy-webpack-plugin');

module.exports = {
  entry: './src/index.ts',
  mode: 'production',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    alias: {
      '@tap-wasm': path.resolve(__dirname, 'node_modules/@tap-wasm'),
    },
  },
  output: {
    filename: 'index.js',
    path: path.resolve(__dirname, 'dist'),
    library: {
      name: 'tapAgent',
      type: 'umd',
    },
    globalObject: 'this',
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        { from: 'node_modules/@tap-wasm/tap_wasm_bg.wasm', to: 'tap_wasm_bg.wasm' },
      ],
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
};