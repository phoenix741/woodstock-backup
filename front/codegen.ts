import type { CodegenConfig } from '@graphql-codegen/cli';

const config: CodegenConfig = {
  schema: 'http://localhost:3000/graphql',
  documents: ['src/**/*.ts', 'src/**/*.vue', 'src/**/*.graphql'],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    'src/generated/': {
      preset: 'client',
      plugins: [
        {
          '@thx/graphql-typescript-scalar-type-policies': {
            scalarTypePolicies: {
              BigInt: '../utils/bigint.utils#bigintTypePolicy',
            },
          },
        },
      ],
      config: {
        useTypeImports: true,
        scalars: {
          BigInt: 'bigint',
        },
      },
    },
    'src/generated/introspection.json': {
      plugins: ['fragment-matcher'],
      config: {
        apolloClientVersion: 3,
      },
    },
  },
};

export default config;
