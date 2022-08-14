import type { IMinimatch } from 'minimatch';
import { Minimatch } from 'minimatch';

export function globStringToRegex(str: string): IMinimatch {
  return new Minimatch(str, { matchBase: true });
}
