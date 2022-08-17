import { FileManifest } from '../shared';

/**
 * Used to represent a manifest in the index manifest.
 */
export class IndexFileEntry {
  markViewed = false;

  constructor(public readonly manifest: FileManifest) {}

  get path(): Buffer {
    return this.manifest.path;
  }
}
