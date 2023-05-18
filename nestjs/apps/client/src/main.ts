import 'source-map-support/register';
import { ServerCredentials } from '@grpc/grpc-js';
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { readFile } from 'fs/promises';
import { resolve } from 'path';
import { AppModule, ConfigModule } from './app.module.js';
import { ClientConfigService } from './client.config.js';
import { LogService } from './log.service.js';
import { Logger } from '@nestjs/common';

async function bootstrap() {
  const appStandalone = await NestFactory.createApplicationContext(ConfigModule, { logger: false });
  const configService = appStandalone.get(ClientConfigService);

  const credentials = ServerCredentials.createSsl(await readFile(configService.rootCA), [
    {
      private_key: await readFile(configService.privateKey),
      cert_chain: await readFile(configService.publicKey),
    },
  ]);

  const logger = new Logger('bootstrap');
  const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'woodstock',
      protoPath: resolve('woodstock.proto'),
      url: configService.config.bind,
      credentials,
    },
    bufferLogs: true,
  });
  app.useLogger(app.get(LogService));

  logger.log(`Starting client on ${configService.config.hostname} (listening on port ${configService.config.bind})`);
  app.listen();
}

bootstrap();
