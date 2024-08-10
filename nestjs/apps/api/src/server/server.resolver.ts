import { Query, Resolver } from '@nestjs/graphql';
import { CommandCheck, CommandCheckFn, ServerChecks, ServerInformations } from './server.dto';
import { ServerService } from './server.service';

@Resolver()
export class ServerResolver {
  constructor(public serverService: ServerService) {}

  @Query(() => [CommandCheck])
  async status(): Promise<CommandCheck[]> {
    const checks = await this.serverService.check();
    const commands = await Promise.all(checks.commands.map((command) => command()));

    return commands;
  }

  @Query(() => ServerInformations)
  informations(): Promise<ServerInformations> {
    return this.serverService.getInformations();
  }
}
