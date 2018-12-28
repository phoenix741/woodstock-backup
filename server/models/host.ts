import * as fs from 'fs'
import * as mkdirp from 'mkdirp'
import * as util from 'util'
import * as yaml from 'js-yaml'
import { NG_MASS_BACKUP_CONFIG_PATH, NG_MASS_BACKUP_CONFIG_HOSTS_PATH } from '../config/index'
import { compact } from '../utils/lodash'
import logger from '../utils/logger'

const mkdirpPromise = util.promisify(mkdirp)

export interface BackupTaskShare {
  name: string
  includes?: Array<string>
  excludes?: Array<string>
}

export interface BackupTaskConfig {
  name: string
  addresses?: Array<string>
  backup: {
    share: Array<BackupTaskShare>
    includes?: Array<string>
    excludes?: Array<string>
    timeout: number
    preUserCommand?: string
    postUserCommand?: string
  }
}

export async function getHosts (): Promise<Array<BackupTaskConfig>> {
  logger.log({ level: 'info', message: `Read the file ${NG_MASS_BACKUP_CONFIG_HOSTS_PATH}` })

  try {
    await mkdirpPromise(NG_MASS_BACKUP_CONFIG_PATH)

    const hostsFromStr = await fs.promises.readFile(NG_MASS_BACKUP_CONFIG_HOSTS_PATH, 'utf8')
    return yaml.safeLoad(hostsFromStr) || []
  } catch (err) {
    logger.log({ level: 'error', message: `Can't read hosts files ${err.message}`, err })
    return []
  }
}

export async function setHosts (hosts: Array<BackupTaskConfig>): Promise<void> {
  logger.log({ level: 'info', message: `Write the file ${NG_MASS_BACKUP_CONFIG_HOSTS_PATH}` })

  try {
    await mkdirpPromise(NG_MASS_BACKUP_CONFIG_PATH)

    const hostsFromStr = yaml.safeDump(compact(hosts))
    await fs.promises.writeFile(NG_MASS_BACKUP_CONFIG_HOSTS_PATH, hostsFromStr, 'utf-8')
  } catch (err) {
    logger.log({ level: 'error', message: `Can't write hosts files: ${err.message}`, err })
  }
}
