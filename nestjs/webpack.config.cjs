const { NormalModuleReplacementPlugin } = require('webpack');

module.exports = function (options) {
  const oldExternals = options.externals;

  return {
    ...options,
    target: 'node16',
    devtool: 'source-map',
    // resolve: {
    //   ...options.resolve,
    //   extensionAlias: {
    //     '.js': ['.ts', '.js'],
    //     '.mjs': ['.mts', '.mjs'],
    //   },
    // },
    output: {
      ...options.output,
      libraryTarget: 'commonjs2',
    },
    externals: [
      function ({ request }, callback) {
        if (request === 'file-type') {
          callback(null, request, 'module');
          return;
        }
        callback();
      },
      ...oldExternals,
    ],
    plugins: [
      new NormalModuleReplacementPlugin(/.js$/, (resource) => {
        if (/node_modules/.test(resource.context)) return;
        resource.request = resource.request.replace(/\.js$/, '');
      }),
      ...options.plugins,
    ],
  };
};
