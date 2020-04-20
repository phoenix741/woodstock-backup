import { Injectable, Logger } from '@nestjs/common';
import * as dns from 'dns';
import * as shell from 'shelljs';
import * as util from 'util';

import { HostConfig } from '../hosts/host-config.dto';

const dnsLookupPromise = util.promisify(dns.lookup);

const NMBLOOKUP_GROUP_ENTRY = /<\w{2}> - <GROUP>/i;
const NMBLOOKUP_ACTIVE_ENTRY = /^\s*([\w\s-]+?)\s*<(\w{2})\> - .*<ACTIVE>/i;

@Injectable()
export class ResolveService {
  private logger = new Logger(ResolveService.name);

  /**
   * Resolve a host from name to IP.
   *
   * Read the configuration file.
   *
   * @param config Configuration of the host
   * @returns The IP of the host
   * @throws If the IP of the host can't be found
   */
  async resolveFromConfig(config: HostConfig): Promise<string> {
    if (config.dhcp) {
      for (const dhcp of config.dhcp) {
        const ip = await this.searchIpFromRange(config.name, dhcp.address, dhcp.start, dhcp.end);
        if (ip) {
          return ip;
        }
      }

      throw new Error(`Can't find any IP for host ${config.name}`);
    } else {
      return this.resolve(config.name, config.addresses || []);
    }
  }

  /**
   * Find the host from a list of address.
   *
   * @param host Hostname from the config file
   * @param addresses Lost of address alternative to the host
   * @returns The IP of the host
   * @throws If the IP of the host can't be found
   */
  async resolve(host: string, addresses: Array<string>): Promise<string> {
    const testableHostname = [host, ...addresses];

    for (const hostname of testableHostname) {
      const ip = await this.findIP(hostname);
      if (ip) {
        return ip;
      }
    }

    throw new Error(`Can't find any IP for host ${host}`);
  }

  /**
   * Search the IP for a host for a given network, from start to end.
   *
   * @param host Hostname from the config file
   * @param network The 3 first number of the IP (that represent the network)
   * @param start Start of the IP to check
   * @param end End of the IP to check
   * @returns The IP of the host
   * @throws If the IP of the host can't be found
   */
  async searchIpFromRange(host: string, network: string, start: number, end: number): Promise<string> {
    for (let n = start; n <= end; n++) {
      const ip = network + '.' + n;
      const { hostname } = await this.resolveNetbiosFromIP(ip);
      if (hostname === host.toLowerCase()) {
        return ip;
      }
    }
    return '';
  }

  async findIP(hostname: string): Promise<string> {
    const ipFromDNS = await this.resolveDNS(hostname);
    if (ipFromDNS) {
      return ipFromDNS;
    }
    return this.resolveNetbiosFromHostname(hostname);
  }

  async resolveDNS(hostname: string): Promise<string> {
    const result = await dnsLookupPromise(hostname);
    return result.address;
  }

  async resolveNetbiosFromHostname(hostname: string): Promise<string> {
    return new Promise(resolve => {
      shell.exec(`nmblookup ${hostname}`, { silent: true }, (code, stdout, stderr) => {
        const result = stdout + '\n' + stderr;

        let subnet = null;
        let firstIpAddr = null;
        let ipAddr = null;

        for (const line of result.split(/[\n\r]/)) {
          const subnetResult = new RegExp(`querying\\s+${hostname}\\s+on\\s+((\\d+\.\\d+\.\\d+)\.(\\d+))`, 'i').exec(line);
          const regexIPResult = new RegExp(`^\\s*(\\d+\\.\\d+\\.\\d+\\.\\d+)\\s+${hostname}`).exec(line);
          if (subnetResult && subnetResult.length) {
            subnet = subnetResult[1];
            if (subnetResult[3] === '255') {
              subnet = subnetResult[2];
            }
          } else if (regexIPResult && regexIPResult.length) {
            const ip = regexIPResult[1];
            if (!firstIpAddr) {
              firstIpAddr = ip;
            }
            if (!ipAddr && subnet && ip.startsWith(subnet)) {
              ipAddr = ip;
            }
          }
        }
        ipAddr = ipAddr || firstIpAddr;

        if (ipAddr) {
          this.logger.log(`Found IP addresse ${ipAddr} for host ${hostname}`);
          resolve(ipAddr);
        } else {
          this.logger.error(`Couldn't find IP addresse for host ${hostname}`);
          resolve();
        }
      });
    });
  }

  async resolveNetbiosFromIP(ip: string): Promise<{ hostname?: string; username?: string }> {
    return new Promise(resolve => {
      shell.exec(`nmblookup -A ${ip}`, { silent: true }, (code, stdout, stderr) => {
        const result = stdout + '\n' + stderr;

        let netBiosHostName;
        let netBiosUserName;
        for (const line of result.split(/[\n\r]/)) {
          const activeEntryResult = NMBLOOKUP_ACTIVE_ENTRY.exec(line);
          if (!line.match(NMBLOOKUP_GROUP_ENTRY) && activeEntryResult && activeEntryResult.length) {
            if (!netBiosHostName) {
              activeEntryResult[2] === '00' && (netBiosHostName = activeEntryResult[1]);
            }
            activeEntryResult[2] === '03' && (netBiosUserName = activeEntryResult[1]);
          }
        }

        if (netBiosHostName) {
          this.logger.log(`Returning host ${netBiosHostName}, user ${netBiosUserName} for ip ${ip}`);
          resolve({ hostname: netBiosHostName.toLowerCase(), username: (netBiosUserName || '').toLowerCase() });
        } else {
          this.logger.error(`Can't find a netbios name for the ip ${ip}`);
          resolve({});
        }
      });
    });
  }
}
