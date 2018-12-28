/**
 * Remove from the object or from the array, all null and undefined value.
 * @param obj Object to clean
 * @returns Cleaned object
 */
export function compact<T> (obj: T | Array<T>): T | Array<T> {
  let copy = JSON.parse(JSON.stringify(obj))
  if (copy instanceof Array) {
    copy = copy.filter(v => !!v)
  }
  return copy
}
