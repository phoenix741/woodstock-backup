import { NestFactory } from '@nestjs/core';
import { MicroserviceOptions, Transport } from '@nestjs/microservices';
import { AppModule } from './app.module';
import { join } from 'path';

async function bootstrap() {
  const app = await NestFactory.createMicroservice<MicroserviceOptions>(
    AppModule,
    {
      transport: Transport.GRPC,
      options: {
        package: 'woodstock',
        protoPath: join(__dirname, 'woodstock.proto'),
        url: '0.0.0.0:3657',
      },
    },
  );
  app.listen(() => console.log('Microservice is listening'));
}
bootstrap();
