import { Module } from '@nestjs/common';
import { ProtobufService } from './protobuf.service';
import { YamlService } from './yaml.service';

@Module({
  providers: [ProtobufService, YamlService],
  exports: [ProtobufService, YamlService],
})
export class InputOutputModule {}
