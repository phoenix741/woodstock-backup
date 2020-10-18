import { FileManifest } from './manifest.model';

export class IndexFileEntry {
  public manifest?: FileManifest;
  public markViewed = false;

  constructor(public path: Buffer) {}
}
