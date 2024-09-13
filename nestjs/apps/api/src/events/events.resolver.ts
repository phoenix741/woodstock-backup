import { Args, Query, Resolver } from '@nestjs/graphql';
import { ApplicationEvent } from './events.dto';
import { EventsService } from './events.service';

@Resolver(() => ApplicationEvent)
export class EventsResolver {
  constructor(private eventsService: EventsService) {}

  @Query(() => [ApplicationEvent])
  events(@Args('firstEvent') firstEvent: Date, @Args('lastEvent') lastEvent: Date) {
    return this.eventsService.listEvents(firstEvent, lastEvent);
  }
}
