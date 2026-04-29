import type { FieldPolicy } from '@apollo/client/cache';

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

export const dateTypePolicy: FieldPolicy<Date, Date | string> = {
  merge: (_, incoming) => {
    if (incoming === null || incoming === undefined) {
      return incoming;
    } else if (incoming instanceof Date) {
      return incoming;
    } else {
      return new Date(incoming);
    }
  },
};
