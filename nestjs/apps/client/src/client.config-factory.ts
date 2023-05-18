import { Injectable } from '@nestjs/common';
import { JwtModuleOptions, JwtOptionsFactory } from '@nestjs/jwt';
import { ClientConfigService } from './client.config';

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
