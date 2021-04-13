import { from, Observable } from 'rxjs';

import { hashBuffer } from '../../utils/lodash.utils';
import { IndexFileEntry } from './index-file-entry.model';
import {
  EntryType,
  FileManifest,
  FileManifestJournalEntryAddOrModify,
  FileManifestJournalEntryRemove,
} from './object-proto.model';

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

  process(journalEntry: FileManifestJournalEntryAddOrModify | FileManifestJournalEntryRemove): void {
    if (journalEntry.type === EntryType.REMOVE) {
      this.remove(journalEntry.path);
    } else {
      this.add(journalEntry.manifest);
    }
  }

  add(manifest: FileManifest): void {
    const key = hashBuffer(manifest.path);
    if (!this.files.has(key)) {
      this.files.set(key, new IndexFileEntry(manifest));
    }
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
}
