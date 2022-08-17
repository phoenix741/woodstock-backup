import { Module } from '@nestjs/common';
import { InputOutputModule } from '../input-output';
import { ManifestService } from './manifest.service';

@Module({
  imports: [InputOutputModule],
  providers: [ManifestService],
  exports: [ManifestService],
})
export class ManifestModule {}
