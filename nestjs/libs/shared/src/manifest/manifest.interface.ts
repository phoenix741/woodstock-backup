import { FileManifest } from '../shared';

export interface ManifestChunk {
  sha256: Buffer;
  manifest: FileManifest;
}
