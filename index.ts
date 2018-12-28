import 'dotenv/config'
import { createTask, launchBackup } from './server/services/backup'
import { BackupTaskConfig } from './server/models/host'

const config: BackupTaskConfig = {
  name: 'pc-ulrich',
  backup: {
    excludes: ['*.bak'],
    timeout: 2000,
    share: [
      {
        name: '/home',
        includes: ['/test'],
        excludes: ['/phoenix']
      },
      {
        name: '/etc'
      }
    ]
  }
}

async function start () {
  const task = await createTask(config)
  await launchBackup(task, task => console.log('progression ' + (task.progression || { percent: 0 }).percent + '% (' + task.state + ')'))
}

start().then(console.log).catch(console.log)
