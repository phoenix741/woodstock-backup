import { Metadata, MetadataValue, ServerReadableStream } from '@grpc/grpc-js';
import { CanActivate, ExecutionContext, Injectable, Logger } from '@nestjs/common';
import { AuthService } from './auth.service';

@Injectable()
export class AuthGuard implements CanActivate {
  #logger = new Logger(AuthGuard.name);

  constructor(private authService: AuthService) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const type = context.getType();

    let header: MetadataValue | undefined;
    if (type === 'rpc') {
      const readable = context.getArgByIndex(0) as ServerReadableStream<unknown, unknown>;
      const metadata = readable.metadata ?? (context.getArgByIndex(1) as Metadata);

      if (!metadata || !(metadata instanceof Metadata)) {
        this.#logger.warn('No metadata on query');
        return false;
      }

      header = metadata.get('X-Session-Id')[0];
    }

    if (!header) {
      this.#logger.warn("Can't get the header");
      return false;
    }

    const token = header.toString();

    try {
      await this.authService.checkContext(token);
      this.#logger.log('Authetification made with success');
      return true;
    } catch (err) {
      this.#logger.error("Can't authenticate with token " + token, err);
      return false;
    }
  }
}
