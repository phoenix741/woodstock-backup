import { constants as constantsFs } from 'fs';
import { access } from 'fs/promises';

export async function isExists(path: string) {
  return access(path, constantsFs.F_OK)
    .then(() => true)
    .catch(() => false);
}
