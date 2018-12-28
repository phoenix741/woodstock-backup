import * as util from 'util'
import { createQueue, Job, Queue } from 'kue'
import { createTask, launchBackup } from '../services/backup'
import { BackupTaskConfig } from '../models/host'
import logger from '../utils/logger'

const jobGetPromise = util.promisify(Job.get)

export class BackupQueue {
  private _queue: Queue
  private _parallele = 1
  private _timeout = 60000

  constructor () {
    this.listen()
  }

  private listen () {
    this._queue = createQueue()
    this._queue
      .on('job enqueue', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been enqueue`))
      .on('job start', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been started`))
      .on('job failed', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been failed`))
      .on('job complete', id => this.logMessageWithName(id, jobName => `The backup of the host ${jobName} have been completed`))
      .on('job progress', (id, progress) => {
        //console.log(progress)
      })
      .on('error', (err: Error) => {
        logger.log({ level: 'error', message: `Can't complete the backup ${err.message}`, err })
      })
      .process('backup', this._parallele, async (job, done) => {
        try {
          const task = await createTask(job.data)
          await launchBackup(task, task => job.progress(task.progression.percent, 100, task))
          done(null, task)
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
    this.shutdown().then(() => this.listen())
  }
}

export default new BackupQueue()
