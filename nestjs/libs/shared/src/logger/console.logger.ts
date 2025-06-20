import { Logger } from '@nestjs/common';
import { JsBackupLogMessage, JsLogLevel, initLog } from '@woodstock/shared-rs';

export function loggerCallback(logger: Logger) {
  return (msg: JsBackupLogMessage) => {
    if (msg.progress) {
      const logMessage = {
        message: msg.progress.message,
      };

      switch (msg.progress.level) {
        case JsLogLevel.Debug:
          logger.debug(logMessage, msg.progress.context);
          break;
        case JsLogLevel.Trace:
          logger.verbose(logMessage, msg.progress.context);
          break;
        case JsLogLevel.Info:
          logger.log(logMessage, msg.progress.context);
          break;
        case JsLogLevel.Warn:
          logger.warn(logMessage, msg.progress.context);
          break;
        case JsLogLevel.Error:
          logger.error(logMessage, msg.progress.context);
          break;
      }
    }
  };
}

export function initializeLog() {
  const logger = new Logger('SharedLogged');

  // This captures the current AsyncLocalStorage context and preserves it
  initLog(loggerCallback(logger));
}
