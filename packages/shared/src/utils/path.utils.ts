import { sep } from 'path';

const SEPARATOR = Buffer.from(sep);

export function joinBuffer(...entries: Buffer[]): Buffer {
  return entries.reduce((prev, next) => {
    if (String.fromCharCode(prev[prev.length - 1]) === sep) {
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

export function hashBuffer(buffer: Buffer): string {
  return buffer.toString('base64');
}

export function getTemporaryFileName(): string {
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}
