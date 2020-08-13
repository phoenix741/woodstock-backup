import { Injectable } from '@nestjs/common';
import { ServeStaticModuleOptions, ServeStaticModuleOptionsFactory } from '@nestjs/serve-static';

import { ApplicationConfigService } from '../config/application-config.service';

@Injectable()
export class ServeStaticService implements ServeStaticModuleOptionsFactory {
  constructor(private configService: ApplicationConfigService) {}

  createLoggerOptions(): ServeStaticModuleOptions[] {
    return [
      {
        rootPath: this.configService.staticPath,
        exclude: ['/graphql'],
      },
    ];
  }
}
