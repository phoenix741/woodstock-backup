import { Minimatch } from 'minimatch';

export function globStringToRegex(str: string): Minimatch {
  return new Minimatch(str, { matchBase: true });
}
