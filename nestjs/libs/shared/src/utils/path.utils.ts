import { sep } from 'path';
import * as tmp from 'tmp';
import { promisify } from 'util';

export const SEPARATOR = Buffer.from(sep);

export const tmpNameAsync = promisify((options: tmp.TmpNameOptions, cb: tmp.TmpNameCallback) =>
  tmp.tmpName(options, cb),
);

export function splitBuffer(buffer: Buffer, delimiter: Buffer = SEPARATOR): Buffer[] {
  let search = -1;
  const result: Buffer[] = [];

  while ((search = buffer.indexOf(delimiter)) > -1) {
    const part = buffer.subarray(0, search);
    if (part.length) {
      result.push(part);
    }
    buffer = buffer.subarray(search + delimiter.length, buffer.length);
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
