import { Module } from '@nestjs/common';
import { CoreModule } from '@woodstock/core';
import { CertificateService, EncryptionService } from './authentification';
import { BackupOnClientService } from './client';
import { ManifestService } from './manifest';
import { FileBrowserService, FileReaderService } from './scanner';

const providers = [
  CertificateService,
  EncryptionService,
  BackupOnClientService,
  ManifestService,
  FileReaderService,
  FileBrowserService,
];

@Module({
  imports: [CoreModule],
  providers,
  exports: providers,
})
export class SharedModule {}
