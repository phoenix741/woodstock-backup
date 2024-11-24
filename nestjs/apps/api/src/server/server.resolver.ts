import { Mutation, Query, Resolver } from '@nestjs/graphql';
import { ClearCacheResponse, ServerInformations } from './server.dto';
import { ServerService } from './server.service';

@Resolver()
export class ServerResolver {
  constructor(public serverService: ServerService) {}

  @Query(() => ServerInformations)
  informations(): Promise<ServerInformations> {
    return this.serverService.getInformations();
  }

  @Mutation(() => ClearCacheResponse)
  async clearCache(): Promise<ClearCacheResponse> {
    await this.serverService.clearCache();
    return new ClearCacheResponse();
  }
}
