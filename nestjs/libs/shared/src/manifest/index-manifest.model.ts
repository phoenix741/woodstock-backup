import { from } from 'ix/iterable';
import { EntryType, FileManifest, FileManifestJournalEntry } from '../models/woodstock';
import { mangle } from '../utils/path.utils';
import { IndexFileEntry } from './index-file-entry.model';

/**
 * List of manifest of a manifest file (and the journal).
 */
export class IndexManifest {
  private files = new Map<string, IndexFileEntry>();

  /**
   * Number of element in the index manifest.
   */
  get indexSize(): number {
    return this.files.size;
  }

  process(journalEntry: FileManifestJournalEntry): void {
    if (journalEntry.type === EntryType.REMOVE && journalEntry.manifest?.path) {
      this.remove(journalEntry.manifest.path);
    } else if (journalEntry.manifest) {
      this.add(journalEntry.manifest);
    }
  }

  add(manifest: FileManifest): void {
    const key = mangle(manifest.path);
    this.files.set(key, new IndexFileEntry(manifest));
  }

  remove(filePath: Buffer): void {
    const key = mangle(filePath);
    this.files.delete(key);
  }

  mark(indexOrPath: IndexFileEntry | Buffer): void {
    const index = isIndexFileEntry(indexOrPath) ? indexOrPath : this.getEntry(indexOrPath);
    if (index) {
      index.markViewed = true;
    }
  }

  /**
   * Read all the file in the walk;
   * @param filter
   */
  walk(): Iterable<IndexFileEntry> {
    return from(this.files.values());
  }

  /**
   * Get the index entry for the given file path;
   */
  getEntry(filePath: Buffer): IndexFileEntry | undefined {
    const key = mangle(filePath);
    return this.files.get(key);
  }
}

function isIndexFileEntry(indexOrPath: IndexFileEntry | Buffer): indexOrPath is IndexFileEntry {
  return (indexOrPath as IndexFileEntry).path !== undefined;
}
