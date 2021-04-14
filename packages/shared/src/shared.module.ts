import { Module } from '@nestjs/common';

import { FileBrowserService } from './file/file-browser.service';
import { FileReaderService } from './file/file-reader.service';

@Module({
  providers: [FileBrowserService, FileReaderService],
})
export class SharedModule {}
