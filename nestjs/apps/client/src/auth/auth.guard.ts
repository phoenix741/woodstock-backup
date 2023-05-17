import { Metadata, MetadataValue } from '@grpc/grpc-js';
import { CanActivate, ExecutionContext, Injectable } from '@nestjs/common';
import { AuthService } from './auth.service';

@Injectable()
export class AuthGuard implements CanActivate {
  constructor(private authService: AuthService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const type = context.getType();

    let header: MetadataValue | undefined;
    if (type === 'rpc') {
      const metadata = context.getArgByIndex(1) as Metadata;
      if (!metadata) {
        return false;
      }
      header = metadata.get('X-Session-Id')[0];
    }

    if (!header) {
      return false;
    }

    const token = header.toString();

    try {
      await this.authService.checkContext(token);
      return true;
    } catch (err) {
      return false;
    }
  }
}
