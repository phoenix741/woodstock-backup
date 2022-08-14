import 'source-map-support/register';
import { ServerCredentials } from '@grpc/grpc-js';
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { readFile } from 'fs/promises';
import { resolve } from 'path';
import { AppModule } from './app.module.js';
import { ClientConfigService } from './config/client.config.js';
import { LogService } from './logger/log.service.js';

async function bootstrap() {
  const appStandalone = await NestFactory.createApplicationContext(AppModule);
  const configService = appStandalone.get(ClientConfigService);

  const credentials = ServerCredentials.createSsl(await readFile(configService.rootCA), [
    {
      private_key: await readFile(configService.privateKey),
      cert_chain: await readFile(configService.publicKey),
    },
  ]);

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
  app.listen();
}

bootstrap();
