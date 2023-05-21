import { ConsoleLogger, Injectable, LogLevel } from '@nestjs/common';
import { LogEntry, LogLevel as GrpcLogLevel } from '@woodstock/shared';
import { Subject } from 'rxjs';

const mapGrpcLogLevel = {
  'log': GrpcLogLevel.log,
  'error': GrpcLogLevel.error,
  'warn': GrpcLogLevel.warn,
  'debug': GrpcLogLevel.debug,
  'verbose': GrpcLogLevel.debug,
  'fatal': GrpcLogLevel.error,
};

@Injectable()
export class LogService extends ConsoleLogger {
  private logStream = new Subject<LogEntry>();

  protected printMessages(
    messages: unknown[],
    context?: string,
    logLevel?: LogLevel,
    writeStreamType?: 'stdout' | 'stderr',
  ): void {
    super.printMessages(messages, context, logLevel, writeStreamType);
    for (const message of messages) {
      this.logStream.next({
        level: logLevel ? mapGrpcLogLevel[logLevel] : GrpcLogLevel.log,
        line: (message as any).toString(),
        context: context || LogService.name,
      });
    }
  }

  public getLogAsObservable() {
    return this.logStream.asObservable();
  }
}
