import { Query, Resolver } from '@nestjs/graphql';
import { ServerInformations } from './server.dto';
import { ServerService } from './server.service';

@Resolver()
export class ServerResolver {
  constructor(public serverService: ServerService) {}

  @Query(() => ServerInformations)
  informations(): Promise<ServerInformations> {
    return this.serverService.getInformations();
  }
}
