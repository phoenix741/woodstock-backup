import { FileManifest } from '../models/woodstock';

/**
 * Used to represent a manifest in the index manifest.
 */
export class IndexFileEntry {
  public markViewed = false;

  constructor(public manifest: FileManifest) {}

  get path(): Buffer {
    return this.manifest.path;
  }
}
