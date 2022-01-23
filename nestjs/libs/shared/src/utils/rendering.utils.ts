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
