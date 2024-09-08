import { Injectable, Logger, OnApplicationShutdown, OnModuleInit } from '@nestjs/common';
import { AbortHandle, CoreClientResolver, JsSocketAddrInformation, resolveDns } from '@woodstock/shared-rs';
import { InformationToResolve } from './resolve.model';

@Injectable()
export class ResolveService implements OnModuleInit, OnApplicationShutdown {
  #logger = new Logger(ResolveService.name);
  #abort?: AbortHandle;

  constructor(private resolver: CoreClientResolver) {}

  onModuleInit() {
    this.#logger.log('Listening for DNS resolution requests');
    this.#abort = this.resolver.listen();
  }

  onApplicationShutdown(signal?: string) {
    this.#logger.log('Shutting down DNS resolution');
    this.#abort?.abort();
  }

  async resolveFromConfig(hostname: string, config?: InformationToResolve): Promise<string[]> {
    if (config?.addresses) {
      this.#logger.debug(`Resolving addresses from configuration for ${hostname}`);

      // If addresses already given in the configuration, we don't need to mdns
      const addresses = await Promise.all(config.addresses.map((address) => resolveDns(address)));
      return addresses
        .flat()
        .filter((address): address is string => address !== null)
        .map((address) => `${address}:${config.port}`);
    }

    this.#logger.debug(`Resolving addresses from mdns for ${hostname}`);
    return (await this.resolver.resolve(hostname, config?.port)) ?? [];
  }

  getInformations(hostname: string): Promise<JsSocketAddrInformation | null> {
    return this.resolver.getInformations(hostname);
  }
}
