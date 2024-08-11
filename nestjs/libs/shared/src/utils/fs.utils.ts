import { constants as constantsFs } from 'fs';
import { access } from 'fs/promises';
import { join, normalize, resolve } from 'path';

export async function isExists(path: string) {
  return access(path, constantsFs.F_OK)
    .then(() => true)
    .catch(() => false);
}

export async function findNearestPackageJson(): Promise<string | undefined> {
  let currentPath = process.cwd();
  while (currentPath !== '/') {
    const packageJson = resolve(currentPath, 'package.json');

    if (await isExists(packageJson)) {
      return packageJson;
    }

    currentPath = normalize(join(currentPath, '..'));
  }

  return undefined;
}
