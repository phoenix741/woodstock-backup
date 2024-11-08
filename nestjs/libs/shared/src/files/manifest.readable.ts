import { CoreFilesService, JsFileManifest } from '@woodstock/shared-rs';
import { Readable } from 'node:stream';

export class CallbackReadable extends Readable {
  private reading: boolean;

  constructor(
    private manifest: JsFileManifest,
    private fileService: CoreFilesService,
  ) {
    super();
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
      } else {
        this.push(null);
      }
    });
  }
}
