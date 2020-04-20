/**
 * Remove from the object or from the array, all null and undefined value.
 * @param obj Object to clean
 * @returns Cleaned object
 */
export function compact<T>(obj: T | Array<T>): T | Array<T> {
  let copy = JSON.parse(JSON.stringify(obj));
  if (copy instanceof Array) {
    copy = copy.filter(v => !!v);
  }
  return copy;
}

/**
 * Pick member of an object.
 *
 * @param obj Object to update
 * @param args List of member to keep
 */
export function pick<T, K extends keyof T>(obj: T, ...keys: K[]): Pick<T, K> {
  return keys
    .map(key => [key, obj[key]])
    .reduce((obj, [key, val]) => Object.assign(obj, { [key as string]: val }), {} as Pick<T, K>);
}
