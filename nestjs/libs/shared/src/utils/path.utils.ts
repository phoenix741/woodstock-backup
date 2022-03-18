import { sep } from 'path';
import * as tmp from 'tmp';
import * as util from 'util';

export const SEPARATOR = Buffer.from(sep);

export const tmpNameAsync = util.promisify((options: tmp.TmpNameOptions, cb: tmp.TmpNameCallback) =>
  tmp.tmpName(options, cb),
);

export function joinBuffer(...entries: Buffer[]): Buffer {
  return entries
    .filter((v) => !!v.length)
    .reduce((prev, next) => {
      if (String.fromCharCode(prev[prev.length - 1]) === sep || String.fromCharCode(next[0]) === sep) {
        return Buffer.concat([prev, next]);
      }
      return Buffer.concat([prev, SEPARATOR, next]);
    });
}

export function splitBuffer(buffer: Buffer, delimiter: Buffer = SEPARATOR): Buffer[] {
  let search = -1;
  const result: Buffer[] = [];

  while ((search = buffer.indexOf(delimiter)) > -1) {
    const part = buffer.slice(0, search);
    if (part.length) {
      result.push(part);
    }
    buffer = buffer.slice(search + delimiter.length, buffer.length);
  }

  if (buffer.length) {
    result.push(buffer);
  }

  return result;
}

export function mangle(buffer: Buffer | string): string {
  return encodeURIComponent(buffer as unknown as string);
}

export function unmangle(path: string): Buffer {
  return Buffer.from(decodeURIComponent(path), 'utf-8');
}

export function getTemporaryFileName(): string {
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}
