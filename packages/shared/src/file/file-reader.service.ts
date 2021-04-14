import { Injectable, Logger } from '@nestjs/common';

@Injectable()
export class FileReaderService {
  private logger = new Logger(FileReaderService.name);
}
