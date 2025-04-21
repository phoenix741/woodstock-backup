import { Injectable } from '@nestjs/common';
import {
  JsEvent,
  JsEventBackupInformation,
  JsEventPoolCleanedInformation,
  JsEventPoolInformation,
  JsHashConversionInformation,
  listEvents,
} from '@woodstock/shared-rs';
import {
  ApplicationEvent,
  EventBackupInformation,
  EventHashConversionInformation,
  EventPoolCleanedInformation,
  EventPoolInformation,
} from './events.dto';

function isJsEventBackupInformation(
  i: JsEventBackupInformation | JsEventPoolInformation | JsEventPoolCleanedInformation | JsHashConversionInformation,
): i is JsEventBackupInformation {
  return (i as JsEventBackupInformation).hostname !== undefined;
}

function isJsEventPoolInformation(
  i: JsEventBackupInformation | JsEventPoolInformation | JsEventPoolCleanedInformation | JsHashConversionInformation,
): i is JsEventPoolInformation {
  return (i as JsEventPoolInformation).fix !== undefined && (i as JsEventPoolInformation).inRefcnt !== undefined;
}

function isJsHashConversionInformation(
  i: JsEventBackupInformation | JsEventPoolInformation | JsEventPoolCleanedInformation | JsHashConversionInformation,
): i is JsHashConversionInformation {
  return (i as JsHashConversionInformation).algorithm !== undefined;
}

function fromInformation(
  information:
    | JsEventBackupInformation
    | JsEventPoolInformation
    | JsEventPoolCleanedInformation
    | JsHashConversionInformation,
): EventBackupInformation | EventPoolInformation | EventPoolCleanedInformation | EventHashConversionInformation {
  if (isJsEventBackupInformation(information)) {
    return new EventBackupInformation(information);
  }
  if (isJsEventPoolInformation(information)) {
    return new EventPoolInformation(information);
  }
  if (isJsHashConversionInformation(information)) {
    return new EventHashConversionInformation(information);
  }
  return new EventPoolCleanedInformation(information);
}

function from(rustEvent: JsEvent): ApplicationEvent {
  return {
    uuid: rustEvent.uuid,
    type: rustEvent.type,
    step: rustEvent.step,
    source: rustEvent.source,
    timestamp: new Date(Number(rustEvent.timestamp * 1000n)),
    errorMessages: rustEvent.errorMessages,
    status: rustEvent.status,
    information: rustEvent.information ? fromInformation(rustEvent.information) : undefined,
  };
}

@Injectable()
export class EventsService {
  async #listEvents(startDate: string, endDate: string): Promise<JsEvent[]> {
    return new Promise((resolve, reject) => {
      listEvents(startDate, endDate, (err, events) => {
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
