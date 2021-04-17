import { Injectable } from '@nestjs/common';
import { LogEntry, LogLevel } from '@woodstock/shared';
import { Subject } from 'rxjs';

@Injectable()
export class LogService {
  private logStream = new Subject<LogEntry>();

  public log(line: string, level: LogLevel.INFO) {
    this.logStream.next({
      level,
      line,
    });
  }

  public getLogAsObservable() {
    return this.logStream.asObservable();
  }
}
