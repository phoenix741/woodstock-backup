import { from, Observable } from 'rxjs';

import { hashBuffer } from '../../utils/lodash';
import { IndexFileEntry } from './index-entry.model';

export class IndexManifest {
  public files = new Map<string, IndexFileEntry>();

  add(filePath: Buffer): IndexFileEntry {
    const key = hashBuffer(filePath);
    let entry = this.files.get(key);
    if (!entry) {
      entry = new IndexFileEntry(filePath);
      this.files.set(key, entry);
    }
    return entry;
  }

  remove(filePath: Buffer): void {
    const key = hashBuffer(filePath);
    this.files.delete(key);
  }

  mark(index: IndexFileEntry): void {
    index.markViewed = true;
  }

  /**
   * Read all the file in the walk;
   * @param filter
   */
  walk(): Observable<IndexFileEntry> {
    return from(this.files.values());
  }

  /**
   * Get the index entry for the given file path;
   */
  getEntry(filePath: Buffer): IndexFileEntry | undefined {
    const key = hashBuffer(filePath);
    return this.files.get(key);
  }

  get indexSize(): number {
    return this.files.size;
  }
}
