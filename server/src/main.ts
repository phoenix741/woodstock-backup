import { ValidationPipe } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';

import { AppModule } from './app.module';
import { ApplicationLogger } from './logger/ApplicationLogger.logger';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    logger: new ApplicationLogger(),
  });
  app.setGlobalPrefix('/api');
  app.useGlobalPipes(new ValidationPipe());

  const options = new DocumentBuilder()
    .setTitle('Woodstock Backup')
    .setDescription('Description of the API of woodstock backup')
    .setVersion('1.0')
    .build();
  const document = SwaggerModule.createDocument(app, options);
  SwaggerModule.setup('api', app, document);

  await app.listen(3000);
}
bootstrap();
