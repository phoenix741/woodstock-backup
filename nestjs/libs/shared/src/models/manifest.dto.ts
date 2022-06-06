import { FileManifest } from './woodstock';

export interface ManifestChunk {
  sha256: Buffer;
  manifest: FileManifest;
}
