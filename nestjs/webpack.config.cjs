const { NormalModuleReplacementPlugin } = require('webpack');

module.exports = function (options) {
  const oldExternals = options.externals;

  return {
    ...options,
    target: 'node18',
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
        if (/shared-rs/.test(request)) {
          callback(null, '@woodstock/shared-rs');
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