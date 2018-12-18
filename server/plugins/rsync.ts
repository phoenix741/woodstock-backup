import * as Rsync from 'rsync'
import { RSyncBackupOptions, BackupContext } from './backups'
import { compact } from '../utils/lodash';

const PROGRESS_XFR = /.*\(xfr#(\d+),\s+\w+-chk=(\d+)\/(\d+)\).*/
const PROGRESS_INFO = /\s+([\d,]+)\s+(\d+)%\s+([\d.]+)(\wB)\/s\s+(\d+:\d{1,2}:\d{1,2})\s*/

const sizes = ['bytes', 'KB', 'MB', 'GB', 'TB', 'PB']

export async function backup (host: string, sharePath: string, destination: string, options: RSyncBackupOptions) {
  const isRsyncVersionGreaterThan31 = true

  const rsync = new Rsync()
    .shell(`/usr/bin/ssh -l ${options.username} -o stricthostkeychecking=no -o userknownhostsfile=/dev/null -o batchmode=yes -o passwordauthentication=no`)
    .flags('vD')
    .set('super')
    .set('recursive')
    .set('protect-args')
    .set('numeric-ids')
    .set('perms')
    .set('owner')
    .set('group')
    .set('times')
    .set('links')
    .set('hard-links')
    .set('delete')
    .set('delete-excluded')
    .set('one-file-system')
    .set('partial')
    .set('stats')
    .set('checksum')
    .set('log-format', 'log: %o %i %B %8U,%8G %9l %f%L')

  if (isRsyncVersionGreaterThan31) {
    rsync.set('info', 'progress2')
  }

  if (options.timeout) {
    rsync.set('timeout', '' + options.timeout)
  }

  if (options.includes.length) {
    rsync.include(options.includes)
  }
  if (options.excludes.length) {
    rsync.exclude(options.excludes)
  }

  rsync.source(`${host}:${sharePath}/`).destination(destination + '/')

  options.callbackLogger({level: 'info', message: `Execute command ${rsync.command()}`, label: sharePath})

  return new Promise((resolve, reject) => {
    const context: BackupContext = { percent: 0, sharePath }
    rsync.execute(
      (error, code, cmd) => {
        const partial = code === 23 || code === 24
        if (error && !partial) {
          return reject(error)
        }
        options.callbackProgress(context)
        resolve(!partial)
      },
      data => processOutput(context, options, data),
      data => processOutput(context, options, data, true)
    )
  })
}

function processOutput (context: BackupContext, options: RSyncBackupOptions, data: any, error = false) {
  data.toString()
    .split(/[\n\r]/)
    .reduce((acc: string[], line: string) => {
      const startLine = line.indexOf('log: ')
      if (startLine > 0) {
        acc.push(line.substring(0, startLine))
        acc.push(line.substring(startLine))
      } else {
        acc.push(line)
      }

      return acc
    }, [])
    .filter((line: string): boolean => !!line)
    .filter((line: string): boolean => {
      if (!line.startsWith('log: ')) {
        const progressionWithXfer = line.match(PROGRESS_XFR)
        if (progressionWithXfer) {
          const [, count,, total] = progressionWithXfer

          Object.assign(context, compact({
            newFileCount: rsyncNumberToInt(count),
            fileCount: rsyncNumberToInt(total)
          }))

          options.callbackProgress(context)
        }

        const progressionWithoutXfer = line.match(PROGRESS_INFO)
        if (progressionWithoutXfer) {
          const [, transferedFileSize, percent, speed, speedUnit] = progressionWithoutXfer

          Object.assign(context, compact({
            newFileSize: rsyncNumberToInt(transferedFileSize),
            percent: rsyncNumberToInt(percent),
            speed: rsyncNumberToInt(speed, speedUnit)
          }))

          options.callbackProgress(context)
          return false
        }

        Object.assign(context, compact({
          fileCount: getValueOfRegex(line, /Number of files:\s+([\d+,.]+)\s+.*/),
          newFileCount: getValueOfRegex(line, /Number of created files:\s+([\d+,.]+)\s+.*/),
          //fileCount: getValueOfRegex(line, /Number of deleted files:.*([\d+,]+)\s+.*/)
          fileSize: getValueOfRegex(line, /Total file size:\s+([\d+,.]+)\s+.*/),
          newFileSize: getValueOfRegex(line, /Total transferred file size:\s+([\d+,.]+)\s+.*/)
        }))
      }

      return true
    })
    .forEach((line: string) => options.callbackLogger({level: error ? 'error' : 'info', message: line, label: context.sharePath}))
}

function rsyncNumberToInt (value: string, unit = 'bytes'): number {
  let numberValue = parseInt(value.replace(/,/g, ''))
  numberValue = numberValue * Math.pow(1024, sizes.indexOf(unit))
  return numberValue
}

function getValueOfRegex (line: string, regex: RegExp): number | undefined {
  const match = line.match(regex)
  if (match) {
    return parseInt(match[1].replace(/,/g, ''))
  }
  return undefined
}
