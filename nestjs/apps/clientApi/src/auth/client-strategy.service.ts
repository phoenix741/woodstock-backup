import { Injectable, Logger, UnauthorizedException } from '@nestjs/common';
import { PassportStrategy } from '@nestjs/passport';
import { HostsService } from '@woodstock/shared';
import { Strategy } from 'passport-client-cert';
import { PeerCertificate } from 'tls';

@Injectable()
export class ClientCertificateStrategy extends PassportStrategy(Strategy) {
  #logger = new Logger(ClientCertificateStrategy.name);

  constructor(private hostService: HostsService) {
    super();
  }

  async validate(clientCert: PeerCertificate): Promise<any> {
    const cn = clientCert.subject.CN;
    this.#logger.debug(`Validating client certificate for CN: ${cn}`);
    if (!cn) {
      throw new UnauthorizedException();
    }

    try {
      await this.hostService.getHost(cn);
    } catch (e) {
      this.#logger.error(`Error while validating client certificate for CN: ${cn}`, e);
      throw new UnauthorizedException();
    }

    return cn;
  }
}
