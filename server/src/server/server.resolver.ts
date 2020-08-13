import { Query, Resolver } from '@nestjs/graphql';

import { ServerChecks } from './server.dto';
import { ServerService } from './server.service';

@Resolver()
export class ServerResolver {
  constructor(private server: ServerService) {}

  @Query(() => ServerChecks)
  async status(): Promise<ServerChecks> {
    return this.server.check();
  }
}
