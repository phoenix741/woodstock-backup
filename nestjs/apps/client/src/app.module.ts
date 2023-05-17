import { forwardRef, Inject, Injectable, Module } from '@nestjs/common';
import { JwtModule, JwtModuleOptions, JwtOptionsFactory } from '@nestjs/jwt';
import { ApplicationConfigService, CoreModule, YamlService } from '@woodstock/core';
import { SharedModule } from '@woodstock/shared';
import { AppController } from './app.controller.js';
import { AppService } from './app.service.js';
import { AuthGuard } from './auth/auth.guard.js';
import { AuthService } from './auth/auth.service.js';
import { ClientConfigService } from './client.config.js';
import { GlobalModule } from './global.module.js';
import { LogService } from './log.service.js';

@Module({
  imports: [CoreModule],
  providers: [ClientConfigService],
  exports: [CoreModule, ClientConfigService],
})
export class ConfigModule {}

@Injectable()
export class ClientJwtModuleFactory implements JwtOptionsFactory {
  constructor(private configService: ClientConfigService) {}

  async createJwtOptions(): Promise<JwtModuleOptions> {
    // On Module Init isn't called at this time
    await this.configService.onModuleInit();

    return {
      secret: this.configService.config.secret,
    };
  }
}

@Module({
  imports: [
    CoreModule,
    SharedModule,
    GlobalModule,
    ConfigModule,
    JwtModule.registerAsync({
      useClass: ClientJwtModuleFactory,
      imports: [ConfigModule],
    }),
  ],
  controllers: [AppController],
  providers: [AppService, AuthService, AuthGuard, LogService],
})
export class AppModule {}
