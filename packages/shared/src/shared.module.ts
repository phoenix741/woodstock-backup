import { Module } from '@nestjs/common';

import { FileBrowserService } from './file/file-browser.service';

@Module({
  providers: [FileBrowserService],
})
export class SharedModule {}
