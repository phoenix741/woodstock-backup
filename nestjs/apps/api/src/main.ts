import 'source-map-support/register';
import { RequestMethod, ValidationPipe } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';
import { ApplicationLogger } from '@woodstock/shared';
import { AppModule } from './app.module.js';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    logger: new ApplicationLogger('main'),
  });
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
    .setTitle('Woodstock Backup')
    .setDescription('Description of the API of woodstock backup')
    .setVersion('1.0')
    .build();
  const document = SwaggerModule.createDocument(app, options);
  SwaggerModule.setup('api', app, document);

  await app.listen(3000);
}
bootstrap();
