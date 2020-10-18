import { sep } from 'path';
import { UnaryFunction, Observable, OperatorFunction, pipe } from 'rxjs';
import { filter } from 'rxjs/operators';

interface CompactableObject {
  [key: string]: any;
}

const SEPARATOR = Buffer.from(sep);

export function compactArray<T>(obj: Array<T>): Array<T> {
  // eslint-disable-next-line @typescript-eslint/no-use-before-define
  return obj.filter((v) => !(v === undefined || v === null)).map((v) => compact(v as any));
}

export function compactObject<T extends CompactableObject>(obj: T): T {
  for (const key in obj) {
    if (!(obj[key] === undefined || obj[key] === null)) {
      obj[key] = obj[key];
    } else {
      delete obj[key];
    }
  }

  return obj;
}

export function compact<T>(obj: T | Array<T>): T | Array<T> {
  if (Array.isArray(obj)) {
    return compactArray(obj);
  } else if (typeof obj === 'object') {
    return compactObject(obj);
  } else {
    return obj;
  }
}

/**
 * Pick member of an object.
 *
 * @param obj Object to update
 * @param args List of member to keep
 */
export function pick<T, K extends keyof T>(obj: T, ...keys: K[]): Pick<T, K> {
  return keys
    .map((key) => [key, obj[key]])
    .reduce((obj, [key, val]) => Object.assign(obj, { [key as string]: val }), {} as Pick<T, K>);
}

export function rendering(string: string, context: Record<string, any>, stack = ''): string {
  return Object.keys(context).reduce(function (accumulator, key) {
    const newStack = stack ? stack + '.' : '';
    const find = '\\$\\{\\s*' + newStack + key + '\\s*\\}';
    const re = new RegExp(find, 'g');

    if (typeof context[key] === 'object') {
      return rendering(accumulator, context[key], newStack + key);
    }
    return accumulator.replace(re, context[key]);
  }, string);
}

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

export function notUndefined<T>(): UnaryFunction<Observable<T | null | undefined>, Observable<T>> {
  return pipe(filter((x) => x != null) as OperatorFunction<T | null | undefined, T>);
}
