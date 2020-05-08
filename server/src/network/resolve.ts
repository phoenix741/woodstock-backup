import { Injectable, Logger } from '@nestjs/common';
import * as dns from 'dns';
import * as util from 'util';

import { HostConfiguration } from '../hosts/host-configuration.dto';
import { ExecuteCommandService } from '../operation/execute-command.service';
import { CommandParameters } from '../server/tools.model';

const dnsLookupPromise = util.promisify(dns.lookup);

const NMBLOOKUP_GROUP_ENTRY = /<\w{2}> - <GROUP>/i;
const NMBLOOKUP_ACTIVE_ENTRY = /^\s*([\w\s-]+?)\s*<(\w{2})\> - .*<ACTIVE>/i;

@Injectable()
export class ResolveService {
  private logger = new Logger(ResolveService.name);

  constructor(private executeCommandService: ExecuteCommandService) {}

  /**
   * Resolve a host from name to IP.
   *
   * Read the configuration file.
   *
   * @param config Configuration of the host
   * @returns The IP of the host
   * @throws If the IP of the host can't be found
   */
  async resolveFromConfig(hostname: string, config: HostConfiguration): Promise<string> {
    if (config.dhcp) {
      for (const dhcp of config.dhcp) {
        const ip = await this.searchIpFromRange(hostname, dhcp.address, dhcp.start, dhcp.end);
        if (ip) {
          return ip;
        }
      }

      throw new Error(`Can't find any IP for host ${hostname}`);
    } else {
      return this.resolve(hostname, config.addresses || []);
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
      const { hostname } = await this.resolveNetbiosFromIP({ ip });
      if (hostname === host.toLowerCase()) {
        return ip;
      }
    }
    return '';
  }

  async findIP(hostname: string): Promise<string | undefined> {
    const ipFromDNS = await this.resolveDNS(hostname);
    if (ipFromDNS) {
      return ipFromDNS;
    }
    return this.resolveNetbiosFromHostname({ hostname });
  }

  async resolveDNS(hostname: string): Promise<string | null> {
    try {
      const result = await dnsLookupPromise(hostname);
      return result.address;
    } catch (err) {
      return null;
    }
  }

  async resolveNetbiosFromHostname(params: CommandParameters): Promise<string | undefined> {
    const { stdout, stderr } = await this.executeCommandService.executeTool('resolveNetbiosFromHostname', params, {
      returnCode: true,
    });
    const result = stdout + '\n' + stderr;

    let subnet = null;
    let firstIpAddr = null;
    let ipAddr = null;

    for (const line of result.split(/[\n\r]/)) {
      const subnetResult = new RegExp(`querying\\s+${params.hostname}\\s+on\\s+((\\d+\.\\d+\.\\d+)\.(\\d+))`, 'i').exec(
        line,
      );
      const regexIPResult = new RegExp(`^\\s*(\\d+\\.\\d+\\.\\d+\\.\\d+)\\s+${params.hostname}`).exec(line);
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
      this.logger.debug(`Found IP addresse ${ipAddr} for host ${params.hostname}`);
      return ipAddr;
    } else {
      this.logger.debug(`Couldn't find IP addresse for host ${params.hostname}`);
      return;
    }
  }

  async resolveNetbiosFromIP(params: CommandParameters): Promise<{ hostname?: string; username?: string }> {
    const { stdout, stderr } = await this.executeCommandService.executeTool('resolveNetbiosFromIP', params, {
      returnCode: true,
    });

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
      this.logger.log(`Returning host ${netBiosHostName}, user ${netBiosUserName} for ip ${params.ip}`);
      return { hostname: netBiosHostName.toLowerCase(), username: (netBiosUserName || '').toLowerCase() };
    } else {
      this.logger.error(`Can't find a netbios name for the ip ${params.ip}`);
      return {};
    }
  }
}
