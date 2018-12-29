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

export interface DhcpAddress {
  address: string
  start: number
  end: number
}

export interface BackupTaskConfig {
  name: string
  addresses?: Array<string>
  dhcp?: Array<DhcpAddress>
  backup: {
    share: Array<BackupTaskShare>
    includes?: Array<string>
    excludes?: Array<string>
    timeout: number
    preUserCommand?: string
    postUserCommand?: string
  }
}

export class Hosts {
  private _hosts: Array<BackupTaskConfig>

  get hosts (): Promise<Array<BackupTaskConfig>> {
    if (! this._hosts) {
      return this.loadHosts().then(() => this._hosts)
    }
    return Promise.resolve(this._hosts)
  }

  private async loadHosts (): Promise<void> {
    logger.log({ level: 'info', message: `Hosts.loadHosts: Read the file ${NG_MASS_BACKUP_CONFIG_HOSTS_PATH}` })

    try {
      await mkdirpPromise(NG_MASS_BACKUP_CONFIG_PATH)

      const hostsFromStr = await fs.promises.readFile(NG_MASS_BACKUP_CONFIG_HOSTS_PATH, 'utf8')
      this._hosts = yaml.safeLoad(hostsFromStr) || []
    } catch (err) {
      this._hosts = []
      logger.log({ level: 'error', message: `Hosts.loadHosts: Can't read hosts files ${err.message}`, err })
    }
  }

  private async writeHosts (): Promise<void> {
    logger.log({ level: 'info', message: `Hosts.writeHosts: Write the file ${NG_MASS_BACKUP_CONFIG_HOSTS_PATH}` })

    try {
      await mkdirpPromise(NG_MASS_BACKUP_CONFIG_PATH)

      const hostsFromStr = yaml.safeDump(compact(this._hosts))
      await fs.promises.writeFile(NG_MASS_BACKUP_CONFIG_HOSTS_PATH, hostsFromStr, 'utf-8')
    } catch (err) {
      logger.log({ level: 'error', message: `Hosts.writeHosts: Can't write hosts files: ${err.message}`, err })
    }
  }
}

export default new Hosts()
