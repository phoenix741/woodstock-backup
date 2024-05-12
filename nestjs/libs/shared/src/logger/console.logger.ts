import { Logger } from '@nestjs/common';
import { BackupContext, LogLevel, initLog } from '@woodstock/shared-rs';

export function initializeLog(context: BackupContext) {
  const logger = new Logger('SharedLogged');
  initLog(context, (msg) => {
    if (msg.progress) {
      switch (msg.progress.level) {
        case LogLevel.Debug:
          logger.debug(msg.progress.message, msg.progress.context);
          break;
        case LogLevel.Trace:
          logger.verbose(msg.progress.message, msg.progress.context);
          break;
        case LogLevel.Info:
          logger.log(msg.progress.message, msg.progress.context);
          break;
        case LogLevel.Warn:
          logger.warn(msg.progress.message, msg.progress.context);
          break;
        case LogLevel.Error:
          logger.error(msg.progress.message, msg.progress.context);
          break;
      }
    }
  });
}
