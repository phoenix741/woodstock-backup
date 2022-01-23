import { IMinimatch, Minimatch } from 'minimatch';

export function globStringToRegex(str: string): IMinimatch {
  return new Minimatch(str, { matchBase: true });
}
