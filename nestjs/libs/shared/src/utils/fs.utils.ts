import { constants as constantsFs } from 'fs';
import { access, unlink } from 'fs/promises';

export async function rm(path: string) {
  try {
    await unlink(path);
  } catch (err) {}
}

export async function isExists(path: string) {
  return access(path, constantsFs.F_OK)
    .then(() => true)
    .catch(() => false);
}
