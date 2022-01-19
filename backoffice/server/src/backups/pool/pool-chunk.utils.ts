import { join } from 'path';

export function calculateChunkDir(poolPath: string, sha256Str: string) {
  const part1 = sha256Str.substring(0, 2);
  const part2 = sha256Str.substring(2, 4);
  const part3 = sha256Str.substring(4, 6);

  return join(poolPath, part1, part2, part3);
}
