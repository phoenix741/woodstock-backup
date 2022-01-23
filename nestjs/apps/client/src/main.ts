import 'source-map-support/register';

import { ServerCredentials } from '@grpc/grpc-js';
import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { readFile } from 'fs/promises';
import { resolve } from 'path';
import { AppModule } from './app.module';
import { LogService } from './logger/log.service';

async function bootstrap() {
  const channel_creds = ServerCredentials.createSsl(await readFile('../client-sync/certs/rootCA.pem'), [
    {
      private_key: await readFile('./certs/server.key'),
      cert_chain: await readFile('./certs/server.crt'),
    },
  ]);

  const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'woodstock',
      protoPath: resolve('woodstock.proto'),
      url: '0.0.0.0:3657',
      credentials: channel_creds,
    },
    bufferLogs: true,
  });
  app.useLogger(app.get(LogService));
  app.listen();
}
bootstrap();
