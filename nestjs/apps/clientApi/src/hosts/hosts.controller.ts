import {
  Body,
  ClassSerializerInterceptor,
  Controller,
  ForbiddenException,
  Param,
  Post,
  UseGuards,
  UseInterceptors,
} from '@nestjs/common';
import { AuthGuard } from '@nestjs/passport';
import { ResolveService } from '@woodstock/shared';
import { CurrentUser } from '../auth/current-user.decorator.js';
import { RegisterClient } from './hosts.dto.js';

@UseInterceptors(ClassSerializerInterceptor)
@Controller('hosts')
export class HostController {
  constructor(private resolveService: ResolveService) {}

  @UseGuards(AuthGuard('client-cert'))
  @Post(':name/client')
  async registerClient(
    @Param('name') name: string,
    @CurrentUser() currentUser: string,
    @Body() body: RegisterClient,
  ): Promise<void> {
    if (currentUser !== name) {
      throw new ForbiddenException(`You are not allowed to register this host: ${name}`);
    }

    const addresses = [
      ...body.addresses.map((address) => address.ipv4?.addr),
      ...body.addresses.map((address) => address.ipv6?.addr),
    ].filter((address) => address !== undefined);

    await this.resolveService.registerInformation({
      hostname: name,
      isOnline: true,
      port: body.port,
      version: body.version,
      addresses,
    });
  }
}
