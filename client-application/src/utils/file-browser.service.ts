import { Injectable, Logger } from '@nestjs/common';
import { FileManifest } from '../app.model';
import { readdir, lstat } from 'fs/promises';

@Injectable()
export class FileBrowser {
  private logger = new Logger(FileBrowser.name);
  /*
  private walkObservable(backupPath: Buffer) {
    return new Observable<FileManifest>((subscribe) => {
      const go = async () => {
        try {
          await this.walk(backupPath, async (manifest) => {
            if (((manifest?.stats?.mode || Long.ZERO).toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFREG) {
              manifest = await this.backupService.readLocalFile(manifest);
            }
            subscribe.next(manifest);
          });
          subscribe.complete();
        } catch (err) {
          subscribe.error(err);
        }
      };

      go();
    });
  }
*/
  private async walk(backupPath: Buffer, progress: (manifest: FileManifest) => Promise<void>) {
    //this.logger.debug(`Read the directory ${backupPath}/${path}`);
    const list = await readdir(backupPath, { encoding: 'buffer' });
    for (const file of list) {
      try {
        const manifestFile = await this.backupService.createManifestFromLocalFile(joinBuffer(backupPath, file));
        await progress(manifestFile);
        if (((manifestFile?.stats?.mode || Long.ZERO).toNumber() & constantsFs.S_IFMT) === constantsFs.S_IFDIR) {
          await this.walk(joinBuffer(backupPath, file), progress);
        }
      } catch (err) {
        this.logger.error(`Can't process the file ${backupPath}: ${err.message}`); // FIXME: Can't process encoding file
      }
    }
  }

  async createManifestFromLocalFile(backupPath: Buffer): Promise<FileManifest> {
    const fileStat = await lstat(backupPath, { bigint: true });
    return {
      path: backupPath,
      stats: {
        ownerId: bigIntToLong(fileStat.uid),
        groupId: bigIntToLong(fileStat.gid),
        size: bigIntToLong(fileStat.size),
        mode: bigIntToLong(fileStat.mode),
        lastModified: bigIntToLong(fileStat.mtimeMs),
        lastRead: bigIntToLong(fileStat.atimeMs),
        created: bigIntToLong(fileStat.birthtimeMs),
      },
    };
  }
}
