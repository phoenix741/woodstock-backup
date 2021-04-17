import { Module } from '@nestjs/common';

import { FileBrowserService } from './file/file-browser.service';
import { FileReader } from './file/file-reader.service';

@Module({
  providers: [FileReader, FileBrowserService],
  exports: [FileReader, FileBrowserService],
})
export class SharedModule {}
