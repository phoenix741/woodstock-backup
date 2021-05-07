import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { readFile } from 'fs/promises';
import { ServerCredentials } from 'grpc';
import { join } from 'path';

import { AppModule } from './app.module';

async function bootstrap() {
  const channel_creds = ServerCredentials.createSsl(await readFile('../client-sync/certs/rootCA.pem'), [
    {
      private_key: await readFile('../client-sync/certs/server.key'),
      cert_chain: await readFile('../client-sync/certs/server.crt'),
    },
  ]);

  const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      package: 'woodstock',
      protoPath: join(__dirname, '..', 'node_modules', '@woodstock', 'shared', 'woodstock.proto'),
      url: '0.0.0.0:3657',
      credentials: channel_creds,
    },
  });
  app.listen(() => console.log('Microservice is listening'));
}
bootstrap();
