import { CoreFilesService, JsFileManifest } from '@woodstock/shared-rs';
import { Readable } from 'node:stream';

export class CallbackReadable extends Readable {
  private manifest: JsFileManifest;
  private fileService: any;
  private reading: boolean;

  constructor(manifest: JsFileManifest, fileService: CoreFilesService) {
    super();
    this.manifest = manifest;
    this.fileService = fileService;
    this.reading = false;
  }

  _read() {
    if (this.reading) return;
    this.reading = true;

    this.fileService.readFile(this.manifest, (err: Error, data: Buffer) => {
      if (err) {
        this.emit('error', err);
        return;
      }
      if (data) {
        this.push(data);
      }
      this.push(null); // Signal end of stream
    });
  }
}
