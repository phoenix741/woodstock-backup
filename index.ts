import 'dotenv/config'
import backupQueue from './server/services/backup-queue'
import { getHosts } from './server/models/host';

async function start () {
  const hosts = await getHosts()

  for (let config of hosts) {
    await backupQueue.addJob(config)
  }
}

start().then(console.log).catch(console.log)
