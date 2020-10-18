import { Module } from '@nestjs/common';

import { FileBrowserService } from './file/file-browser.service';
import { FileReader } from './file/file-reader.service';
import { ManifestService } from './manifest/manifest.service';

@Module({
  providers: [FileReader, FileBrowserService, ManifestService],
  exports: [FileReader, FileBrowserService, ManifestService],
})
export class SharedModule {}
