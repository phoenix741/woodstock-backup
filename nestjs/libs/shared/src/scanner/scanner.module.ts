import { Module } from '@nestjs/common';
import { FileBrowserService } from './file-browser.service';
import { FileReaderService } from './file-reader.service';

@Module({
  providers: [FileReaderService, FileBrowserService],
  exports: [FileReaderService, FileBrowserService],
})
export class ScannerModule {}
