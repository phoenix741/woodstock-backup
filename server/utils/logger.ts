import * as path from 'path'
import * as mkdirp from 'mkdirp'
import * as util from 'util'
import { createLogger, format, transports, Logger, LogEntry } from 'winston'
import * as logform from 'logform'

import { NG_MASS_BACKUP_LOG_PATH } from '../config/index'

const mkdirpPromise = util.promisify(mkdirp)
const { combine, timestamp, printf } = format

const applicationFormat = printf((info: logform.TransformableInfo) => {
  return `${info.timestamp} ${info.level}: ${info.message}`
})

export class ApplicationLogger {
  private logger: Logger

  constructor () {
    this.logger = createLogger({
      level: 'info',
      format: combine(
        timestamp(),
        applicationFormat
      ),
      transports: [
        new transports.Console({}),
        new transports.File({ filename: path.join(NG_MASS_BACKUP_LOG_PATH, 'application.log') })
      ],
      exceptionHandlers: [
        new transports.File({ filename: path.join(NG_MASS_BACKUP_LOG_PATH, 'exceptions.log') })
      ]
    })
  }

  static async init () {
    await mkdirpPromise(NG_MASS_BACKUP_LOG_PATH)
  }

  log (infoObject: LogEntry) {
    this.logger.log(infoObject)
  }
}

ApplicationLogger.init()

export default new ApplicationLogger()
