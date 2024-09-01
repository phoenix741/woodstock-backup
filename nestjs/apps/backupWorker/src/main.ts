import 'source-map-support/register';

import { NestFactory } from '@nestjs/core';
import { ApplicationLogger } from '@woodstock/shared';

import { AppModule } from './app.module.js';

async function bootstrap() {
  const app = await NestFactory.createApplicationContext(AppModule, {
    bufferLogs: true,
  });
  app.useLogger(app.get(ApplicationLogger));
  app.flushLogs();
}
bootstrap();
