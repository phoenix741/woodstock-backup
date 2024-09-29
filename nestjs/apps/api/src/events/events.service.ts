import { Injectable } from '@nestjs/common';
import { ApplicationConfigService } from '@woodstock/shared';
import {
  JsEvent,
  JsEventBackupInformation,
  JsEventPoolCleanedInformation,
  JsEventPoolInformation,
  JsEventRefCountInformation,
  JsEventSource,
  JsEventStatus,
  JsEventStep,
  JsEventType,
  listEvents,
} from '@woodstock/shared-rs';
import {
  ApplicationEvent,
  EventBackupInformation,
  EventPoolCleanedInformation,
  EventPoolInformation,
  EventRefCountInformation,
  EventSource,
  EventStatus,
  EventStep,
  EventType,
} from './events.dto';

const rustEventTypeToEventTypeMap: Record<JsEventType, EventType> = {
  [JsEventType.Backup]: EventType.Backup,
  [JsEventType.Restore]: EventType.Restore,
  [JsEventType.Delete]: EventType.Delete,
  [JsEventType.RefcntChecked]: EventType.RefcntChecked,
  [JsEventType.PoolChecked]: EventType.PoolChecked,
  [JsEventType.ChecksumChecked]: EventType.ChecksumChecked,
  [JsEventType.PoolCleaned]: EventType.PoolCleaned,
};

const rustEventStepToEventStepMap: Record<JsEventStep, EventStep> = {
  [JsEventStep.Start]: EventStep.Start,
  [JsEventStep.End]: EventStep.End,
};

const rustEventSourceToEventSourceMap: Record<JsEventSource, EventSource> = {
  [JsEventSource.User]: EventSource.User,
  [JsEventSource.Woodstock]: EventSource.Woodstock,
  [JsEventSource.Import]: EventSource.Import,
  [JsEventSource.Cli]: EventSource.Cli,
};

const rustEventStatusToEventStatusMap: Record<JsEventStatus, EventStatus> = {
  [JsEventStatus.None]: EventStatus.None,
  [JsEventStatus.Success]: EventStatus.Success,
  [JsEventStatus.ClientDisconnected]: EventStatus.ClientDisconnected,
  [JsEventStatus.ServerCrashed]: EventStatus.ServerCrashed,
  [JsEventStatus.GenericError]: EventStatus.GenericError,
};

function isJsEventBackupInformation(
  i: JsEventBackupInformation | JsEventRefCountInformation | JsEventPoolInformation | JsEventPoolCleanedInformation,
): i is JsEventBackupInformation {
  return (i as JsEventBackupInformation).hostname !== undefined;
}

function isJsEventRefCountInformation(
  i: JsEventBackupInformation | JsEventRefCountInformation | JsEventPoolInformation | JsEventPoolCleanedInformation,
): i is JsEventRefCountInformation {
  return (i as JsEventRefCountInformation).fix !== undefined && (i as JsEventRefCountInformation).count !== undefined;
}

function isJsEventPoolInformation(
  i: JsEventBackupInformation | JsEventRefCountInformation | JsEventPoolInformation | JsEventPoolCleanedInformation,
): i is JsEventPoolInformation {
  return (i as JsEventPoolInformation).fix !== undefined && (i as JsEventPoolInformation).inRefcnt !== undefined;
}

function fromInformation(
  information:
    | JsEventBackupInformation
    | JsEventRefCountInformation
    | JsEventPoolInformation
    | JsEventPoolCleanedInformation,
): EventBackupInformation | EventRefCountInformation | EventPoolInformation | EventPoolCleanedInformation {
  if (isJsEventBackupInformation(information)) {
    return new EventBackupInformation({
      hostname: information.hostname,
      number: Number(information.number),
      sharePath: information.sharePath,
    });
  }
  if (isJsEventRefCountInformation(information)) {
    return new EventRefCountInformation({
      fix: information.fix,
      count: Number(information.count),
      error: Number(information.error),
    });
  }
  if (isJsEventPoolInformation(information)) {
    return new EventPoolInformation({
      fix: information.fix,
      inUnused: Number(information.inUnused),
      inRefcnt: Number(information.inRefcnt),
      inNothing: Number(information.inNothing),
      missing: Number(information.missing),
    });
  }
  return new EventPoolCleanedInformation({
    size: Number(information.size),
    count: Number(information.count),
  });
}

function from(rustEvent: JsEvent): ApplicationEvent {
  return {
    uuid: rustEvent.uuid,
    type: rustEventTypeToEventTypeMap[rustEvent.type],
    step: rustEventStepToEventStepMap[rustEvent.step],
    source: rustEventSourceToEventSourceMap[rustEvent.source],
    timestamp: new Date(Number(rustEvent.timestamp * 1000n)),
    errorMessages: rustEvent.errorMessages,
    status: rustEventStatusToEventStatusMap[rustEvent.status],
    information: rustEvent.information ? fromInformation(rustEvent.information) : undefined,
  };
}

@Injectable()
export class EventsService {
  constructor(private applicationConfig: ApplicationConfigService) {}

  async #listEvents(startDate: string, endDate: string): Promise<JsEvent[]> {
    return new Promise((resolve, reject) => {
      listEvents(startDate, endDate, this.applicationConfig.context, (err, events) => {
        if (err) {
          reject(err);
        } else {
          resolve(events);
        }
      });
    });
  }

  async listEvents(startDate: Date, endDate: Date): Promise<ApplicationEvent[]> {
    // Convert dates to ISO strings
    const isoStartDate = startDate.toISOString().replace(/T.*/, '');
    const isoEndDate = endDate.toISOString().replace(/T.*/, '');

    const events = await this.#listEvents(isoStartDate, isoEndDate);

    // Sort events by timestamp, return last event first
    events.sort((a, b) => Number(b.timestamp) - Number(a.timestamp));

    return events.map(from);
  }
}
