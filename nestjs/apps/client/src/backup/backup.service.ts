import { BadRequestException, Injectable, UnauthorizedException } from '@nestjs/common';
import { AuthenticateRequest, FileReaderService, ManifestService } from '@woodstock/shared';
import { v4 as uuidv4 } from 'uuid';
import { BackupContext } from './backup-context.class.js';

@Injectable()
export class BackupService {
  private context = new Map<string, BackupContext>();

  constructor(private fileReader: FileReaderService, private manifestService: ManifestService) {}

  initializeBackup(request: AuthenticateRequest) {
    if (request.version !== 0) {
      throw new BadRequestException('Unsupported version');
    }

    const uuid = uuidv4();
    const context = new BackupContext(this.fileReader, this.manifestService);

    this.context.set(uuid, context);

    return uuid;
  }

  getContext(sessionId: string) {
    if (!this.context.has(sessionId)) {
      throw new UnauthorizedException('Session not found');
    }
    return this.context.get(sessionId);
  }
}
