import 'dotenv/config'
import backupQueue from './server/services/backup-queue'
import hostsList from './server/models/host'

async function start () {
  const hosts = await hostsList.hosts

  for (let config of hosts) {
    await backupQueue.addJob(config)
  }
}

start().catch(console.log)
