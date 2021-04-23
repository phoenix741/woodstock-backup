import { Injectable, Logger } from '@nestjs/common';
import { bigIntToLong, FileManifest, CHUNK_SIZE } from '@woodstock/shared';
import * as crypto from 'crypto';
import * as fs from 'fs';

@Injectable()
export class BackupService {
  private logger = new Logger(BackupService.name);

  async createManifestFromLocalFile(backupPath: Buffer): Promise<FileManifest> {
    const fileStat = await fs.promises.lstat(backupPath, { bigint: true });
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

  async readLocalFile(manifest: FileManifest): Promise<FileManifest> {
    //this.logger.debug(`Calculate the checksum of ${backupPath}/${manifest.path}`);
    return new Promise((resolve, reject) => {
      const shasum = crypto.createHash('sha3-256');

      let bufferLength = 0;
      let chunkShasum = crypto.createHash('sha3-256');

      const chunks: Buffer[] = [];
      try {
        const s = fs.createReadStream(manifest.path);
        s.on('data', (data) => {
          const chunkSizeRest = CHUNK_SIZE - bufferLength;
          let shaData, shaDataRest;
          if (data.length >= chunkSizeRest) {
            shaData = data.slice(0, chunkSizeRest);
            shaDataRest = data.slice(chunkSizeRest);
          } else {
            shaData = data;
            shaDataRest = Buffer.alloc(0);
          }
          chunkShasum.update(shaData);
          bufferLength += data.length;

          if (bufferLength >= CHUNK_SIZE) {
            chunks.push(chunkShasum.digest());
            chunkShasum = crypto.createHash('sha3-256');
            chunkShasum.update(shaDataRest);
            bufferLength = shaDataRest.length;
          }

          shasum.update(data);
        });

        s.on('end', () => {
          if (bufferLength) {
            chunks.push(chunkShasum.digest());
          }

          const sha256 = shasum.digest();
          //this.logger.debug(`Hash of the file ${backupPath}/${manifest.path} is : ${sha256.toString('hex')}`);
          manifest.chunks = chunks;
          manifest.sha256 = sha256;
          return resolve(manifest);
        });

        s.on('error', (err) => {
          return reject(err);
        });
      } catch (error) {
        this.logger.error(`Can't calculate the manifest of the file ${manifest.path} : ${error.message}`);
        return reject(error);
      }
    });
  }
}
