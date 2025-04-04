import 'source-map-support/register';

import { RequestMethod, ValidationPipe } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';
import { ApplicationConfigService, ApplicationLogger } from '@woodstock/shared';
import * as express from 'express';
import * as https from 'https';

import { AppModule } from './app.module.js';
import { readFile } from 'node:fs/promises';
import { ExpressAdapter } from '@nestjs/platform-express';
import { join } from 'node:path';
import { ServerOptions } from 'node:https';

async function bootstrap() {
  const server = express();
  const app = await NestFactory.create(AppModule, new ExpressAdapter(server), { bufferLogs: true });
  app.useLogger(app.get(ApplicationLogger));
  app.flushLogs();

  app.setGlobalPrefix('/api', { exclude: [{ path: 'metrics', method: RequestMethod.GET }] });
  app.useGlobalPipes(
    new ValidationPipe({
      transform: true,
      forbidUnknownValues: true,
      skipMissingProperties: true,
      skipNullProperties: true,
      skipUndefinedProperties: true,
    }),
  );

  const options = new DocumentBuilder()
    .setTitle('Woodstock Backup Client API')
    .setDescription('Description of the API of woodstock backup')
    .setVersion('1.0')
    .build();
  const document = SwaggerModule.createDocument(app, options);
  SwaggerModule.setup('api', app, document);

  // Wait for the application to be ready
  await app.init();

  const config = app.get(ApplicationConfigService);
  const httpsOptions: ServerOptions = {
    requestCert: true,
    rejectUnauthorized: false,

    ca: await readFile(join(config.certificatePath, 'rootCA.pem')),
    key: await readFile(join(config.certificatePath, 'https.key')),
    cert: await readFile(join(config.certificatePath, 'https.pem')),
  };
  await https.createServer(httpsOptions, server).listen(config.clientApiPort);
}
bootstrap();
