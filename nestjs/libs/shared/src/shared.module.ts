import { Module } from '@nestjs/common';
import { FileBrowserService } from './file/file-browser.service';
import { FileReader } from './file/file-reader.service';
import { ManifestService } from './manifest/manifest.service';
import { RefCntService } from './refcnt';
import { ProtobufService } from './services/protobuf.service';
import { YamlService } from './services/yaml.service';

@Module({
  providers: [FileReader, FileBrowserService, ManifestService, YamlService, ProtobufService, RefCntService],
  exports: [FileReader, FileBrowserService, ManifestService, YamlService, ProtobufService, RefCntService],
})
export class SharedModule {}
