import { FileManifest } from '../protobuf/woodstock.interface';

export interface ManifestChunk {
  sha256: Buffer;
  manifest: FileManifest;
}
