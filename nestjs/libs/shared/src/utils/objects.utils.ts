export function compactArray<T>(obj: Array<T>): Array<T> {
  // eslint-disable-next-line @typescript-eslint/no-use-before-define
  return obj.filter((v) => !(v === undefined || v === null)).map((v) => compact(v as any));
}

export function compactObject<T extends object>(obj: T): T {
  for (const key in obj) {
    if (!(obj[key] === undefined || obj[key] === null)) {
      obj[key] = obj[key];
    } else {
      delete obj[key];
    }
  }

  return obj;
}

export function compact<T extends object>(obj: T | Array<T>): T | Array<T> {
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
