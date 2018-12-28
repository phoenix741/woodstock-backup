import * as dns from 'dns'

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

async function findIP (hostname: string): Promise<string> {
  return resolveDNS(hostname)
}

async function resolveDNS (hostname: string): Promise<string> {
  const result = await dns.promises.lookup(hostname)
  return result.address
}

async function resolveNetbios (hostname: string): Promise<string> {
  // exec nmblookup <host>
  // failed: name_query failed to find name pc-m-ulrich
  // success: 192.168.176.1 pc-ulrich<00>
}
