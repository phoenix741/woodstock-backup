import { ServerCredentials } from '@grpc/grpc-js';
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { readFile } from 'fs/promises';
import { resolve } from 'path';
import 'source-map-support/register';
import { AppModule } from './app.module';
import { ClientConfigService } from './config/client.config';
import { LogService } from './logger/log.service';

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
