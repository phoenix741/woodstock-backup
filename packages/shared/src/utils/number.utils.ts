import * as Long from 'long';

declare global {
  interface BigInt {
    toJSON(): string;
  }
}

BigInt.prototype.toJSON = function () {
  return this.toString();
};

export function bigIntMax(...args: bigint[]): bigint {
  return args.reduce((m, e) => (e > m ? e : m));
}

export function bigIntMin(...args: bigint[]): bigint {
  return args.reduce((m, e) => (e < m ? e : m));
}

export function longMax(...args: Long[]): Long {
  return args.reduce((m, e) => (e.greaterThan(m) ? e : m));
}

export function longMin(...args: Long[]): Long {
  return args.reduce((m, e) => (e.lessThan(m) ? e : m));
}

export function bigIntToLong(n: bigint): Long {
  return Long.fromString(n.toString());
}

export function longToBigInt(n: Long): bigint {
  return BigInt(n.toString());
}
