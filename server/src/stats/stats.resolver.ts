import { Query, Resolver } from '@nestjs/graphql';

import { Statistics } from './stats.model';
import { StatsService } from './stats.service';

@Resolver(() => Statistics)
export class StatsResolver {
  constructor(private statsService: StatsService) {}

  @Query(() => Statistics)
  async statistics() {
    return this.statsService.getStatistics();
  }

  
}
