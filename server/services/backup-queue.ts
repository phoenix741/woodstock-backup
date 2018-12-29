import * as util from 'util'
import { createQueue, Job, Queue } from 'kue'
import { BackupTaskConfig } from '../models/host'
import { BackupTask } from './backup-task'
import logger from '../utils/logger'

const jobGetPromise = util.promisify(Job.get)

export class BackupQueue {
  private _queue: Queue
  private _parallele = 1
  private _timeout = 60000

  constructor () {
    this.listen()
  }

  async addJob (config: BackupTaskConfig): Promise<void> {
    return new Promise((resolve, reject) => {
      this._queue.create('backup', config).save((err: Error) => {
        if (err) {
          return reject(err)
        }
        resolve()
      })
    })
  }

  set parallele (parallele: number) {
    this._parallele = parallele
    this.shutdown().finally(() => this.listen()).catch(err => logger.log({ level: 'error', message: 'Can\' restart worker that make backup', err }))
  }

  private listen () {
    this._queue = createQueue()
    this._queue
      .on('job enqueue', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been enqueue`))
      .on('job start', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been started`))
      .on('job failed', (id, err) => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been failed: "${err}"`))
      .on('job complete', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been completed`))
      .on('job progress', (id, progress) => {
        // console.log(progress)
      })
      .on('error', (err: Error) => {
        logger.log({ level: 'error', message: `Can't complete the backup ${err.message}`, err })
      })
      .process('backup', this._parallele, async (job, done) => {
        try {
          const task: BackupTask = await BackupTask.createFromHostConfig(job.data)
          try {
            await task.launchBackup(task => job.progress(task.progression.percent, 100, task.toJSON()))
            done(null, task.toJSON())
          } catch (err) {
            done(err, task && task.toJSON())
          }
        } catch (err) {
          done(err)
        }
      })
  }

  private async logMessageWithName (id: number, message: ((jobName: string) => string)) {
    try {
      const job = await jobGetPromise(id)
      job && logger.log({ level: 'info', message: message(job.data.name) })
    } catch (err) {
      logger.log({ level: 'error', message: `Can't get the job ${id}` })
    }
  }

  private async shutdown () {
    return new Promise((resolve, reject) => {
      this._queue.shutdown(this._timeout, (err: Error) => {
        if (err) {
          return reject(err)
        }
        resolve()
      })
    })
  }
}

export default new BackupQueue()
