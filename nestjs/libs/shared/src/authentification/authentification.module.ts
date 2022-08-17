import { Module } from '@nestjs/common';
import { ApplicationConfigModule } from '../config';
import { CertificateService } from './certificate.service';
import { EncryptionService } from './encryption.service';

@Module({
  imports: [ApplicationConfigModule],
  providers: [CertificateService, EncryptionService],
  exports: [CertificateService, EncryptionService],
})
export class AuthentificationModule {}
