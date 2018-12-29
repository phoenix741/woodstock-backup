import * as shell from 'shelljs'
import * as dns from 'dns'
import * as util from 'util'

import logger from '../utils/logger'
import { BackupTaskConfig } from '../models/host'

const dnsLookupPromise = util.promisify(dns.lookup)

export async function resolveFromConfig (config: BackupTaskConfig) {
  if (config.dhcp) {
    for (let dhcp of config.dhcp) {
      const ip = await searchIpFromRange(config.name, dhcp.address, dhcp.start, dhcp.end)
      if (ip) {
        return ip
      }
    }

    throw new Error(`Can't find any IP for host ${config.name}`)
  } else {
    return resolve(config.name, config.addresses || [])
  }
}

export async function resolve (host: string, addresses: Array<string>): Promise<string> {
  const testableHostname = [host, ...addresses]

  for (let hostname of testableHostname) {
    const ip = await findIP(hostname)
    if (ip) {
      return ip
    }
  }

  throw new Error(`Can't find any IP for host ${host}`)
}

export async function searchIpFromRange (host: string, network: string, start: number, end: number): Promise<string> {
  for (let n = start; n <= end; n++) {
    const ip = network + '.' + n
    const { hostname } = await resolveNetbiosFromIP(ip)
    if (hostname === host.toLowerCase()) {
      return ip
    }
  }
  return ''
}

async function findIP (hostname: string): Promise<string> {
  const ipFromDNS = await resolveDNS(hostname)
  if (ipFromDNS) {
    return ipFromDNS
  }
  return resolveNetbiosFromHostname(hostname)
}

async function resolveDNS (hostname: string): Promise<string> {
  const result = await dnsLookupPromise(hostname)
  return result.address
}

async function resolveNetbiosFromHostname (hostname: string): Promise<string> {
  return new Promise(resolve => {
    shell.exec(`nmblookup ${hostname}`, (code, stdout, stderr) => {
      const result = stdout + '\n' + stderr

      let subnet = null
      let firstIpAddr = null
      let ipAddr = null

      for (let line of result.split(/[\n\r]/)) {
        const subnetResult = new RegExp(`querying\\s+${hostname}\\s+on\\s+((\\d+\.\\d+\.\\d+)\.(\\d+))`, 'i').exec(line)
        const regexIPResult = new RegExp(`^\\s*(\\d+\\.\\d+\\.\\d+\\.\\d+)\\s+${hostname}`).exec(line)
        if (subnetResult && subnetResult.length) {
          subnet = subnetResult[1]
          if (subnetResult[3] === '255') {
            subnet = subnetResult[2]
          }
        } else if (regexIPResult && regexIPResult.length) {
          const ip = regexIPResult[1]
          if (!firstIpAddr) {
            firstIpAddr = ip
          }
          if (!ipAddr && subnet && ip.startsWith(subnet)) {
            ipAddr = ip
          }
        }
      }
      ipAddr = ipAddr || firstIpAddr

      if (ipAddr) {
        logger.log({ level: 'info', message: `Found IP addresse ${ipAddr} for host ${hostname}` })
        resolve(ipAddr)
      } else {
        logger.log({ level: 'error', message: `Couldn't find IP addresse for host ${hostname}` })
        resolve()
      }
    })
  })
}

const NMBLOOKUP_GROUP_ENTRY = /<\w{2}> - <GROUP>/i
const NMBLOOKUP_ACTIVE_ENTRY = /^\s*([\w\s-]+?)\s*<(\w{2})\> - .*<ACTIVE>/i

async function resolveNetbiosFromIP (ip: string): Promise<{hostname?: string, username?: string}> {
  return new Promise(resolve => {
    shell.exec(`nmblookup -A ${ip}`, (code, stdout, stderr) => {
      const result = stdout + '\n' + stderr

      let netBiosHostName
      let netBiosUserName
      for (let line of result.split(/[\n\r]/)) {
        const activeEntryResult = NMBLOOKUP_ACTIVE_ENTRY.exec(line)
        if (!line.match(NMBLOOKUP_GROUP_ENTRY) && activeEntryResult && activeEntryResult.length) {
          if (! netBiosHostName) {
            activeEntryResult[2] === '00' && (netBiosHostName = activeEntryResult[1])
          }
          activeEntryResult[2] === '03' && (netBiosUserName = activeEntryResult[1])
        }
      }

      if (netBiosHostName) {
        logger.log({ level: 'info', message: `Returning host ${netBiosHostName}, user ${netBiosUserName} for ip ${ip}` })
        resolve({ hostname: netBiosHostName.toLowerCase(), username: (netBiosUserName || '').toLowerCase() })
      } else {
        logger.log({ level: 'error', message: `Can't find a netbios name for the ip ${ip}` })
        resolve({})
      }
    })
  })
}
