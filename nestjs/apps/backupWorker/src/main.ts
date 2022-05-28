import 'source-map-support/register';
import { NestFactory } from '@nestjs/core';
import { ApplicationLogger } from '@woodstock/shared';
import { AppModule } from './app.module';

async function bootstrap() {
  await NestFactory.createApplicationContext(AppModule, {
    logger: new ApplicationLogger('stats'),
  });
}
bootstrap();
