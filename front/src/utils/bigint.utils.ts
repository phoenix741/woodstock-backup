import { FieldPolicy } from '@apollo/client/cache';

export const bigintTypePolicy: FieldPolicy<bigint, string> = {
  merge: (_, incoming) => {
    if (incoming === null || incoming === undefined) {
      return incoming;
    } else if (typeof incoming === 'bigint') {
      return incoming;
    } else {
      return BigInt(incoming);
    }
  },
};
