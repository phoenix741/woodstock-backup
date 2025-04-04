import { Injectable, Logger } from '@nestjs/common';
import { randomBytes } from 'crypto';
import { mkdir, readFile, writeFile } from 'fs/promises';
import type { pki as PKI } from 'node-forge';
import { md, pki } from 'node-forge';
import { join } from 'path';

import { ApplicationConfigService } from '../config';
import { isExists } from '../utils';

const CERTIFICATE_ATTRS = [
  {
    name: 'countryName',
    value: 'FR',
  },
  {
    shortName: 'ST',
    value: 'Paris',
  },
  {
    name: 'localityName',
    value: 'Paris',
  },
  {
    name: 'organizationName',
    value: 'Woodstock Backup',
  },
  {
    shortName: 'OU',
    value: 'Woodstock Backup',
  },
];

/**
 * Function to check if a string is an IP address (IPv4 or IPv6)
 * @param hostname the hostname or IP to check
 * @return true if the hostname is an IP address
 */
export function isIp(hostname: string): boolean {
  const ipv4Regex =
    /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;

  const ipv6Regex =
    /^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)))$/;

  return ipv4Regex.test(hostname) || ipv6Regex.test(hostname);
}

@Injectable()
export class CertificateService {
  #logger = new Logger(CertificateService.name);

  constructor(private config: ApplicationConfigService) {}

  #createCertificate(
    host: string,
    server: boolean,
    rootCA?: { privateKey: PKI.PrivateKey; certificate: PKI.Certificate },
  ): { privateKey: string; publicKey: string } {
    // generate a keypair and create an X.509v3 certificate
    const keys = pki.rsa.generateKeyPair(2048);
    const cert = pki.createCertificate();

    cert.publicKey = keys.publicKey;
    if (!rootCA) {
      cert.privateKey = keys.privateKey;
    }
    cert.serialNumber = '01' + randomBytes(19).toString('hex');
    cert.validity.notBefore = new Date();
    cert.validity.notAfter = new Date();
    cert.validity.notAfter.setFullYear(cert.validity.notBefore.getFullYear() + 10);
    const attrs = [
      {
        name: 'commonName',
        value: host,
      },
      ...CERTIFICATE_ATTRS,
    ];
    cert.setSubject(attrs);

    if (rootCA) {
      const subjectAltName = isIp(host)
        ? [
            {
              type: 7, // 7 is IP type
              ip: host,
            },
          ]
        : [
            {
              type: 2, // 2 is DNS type
              value: host,
            },
          ];

      cert.setIssuer(rootCA.certificate.subject.attributes);
      const extKeyUsage = server
        ? [
            {
              name: 'keyUsage',
              digitalSignature: true,
              keyEncipherment: true,
            },
            {
              name: 'extKeyUsage',
              serverAuth: true,
            },
            {
              name: 'subjectAltName',
              altNames: subjectAltName,
            },
          ]
        : [
            {
              name: 'extKeyUsage',
              clientAuth: true,
            },
          ];
      cert.setExtensions([
        {
          name: 'basicConstraints',
          cA: false,
        },
        ...extKeyUsage,
        {
          name: 'authorityKeyIdentifier',
          // authorityCertIssuer: true,
          // serialNumber: rootCA.certificate.serialNumber,
          keyIdentifier: rootCA.certificate.generateSubjectKeyIdentifier().getBytes(),
        },
      ]);
      cert.sign(rootCA.privateKey, md.sha256.create());
    } else {
      cert.setIssuer(attrs);
      cert.setExtensions([
        {
          name: 'basicConstraints',
          cA: true,
        },
        {
          name: 'keyUsage',
          keyCertSign: true,
        },
        {
          name: 'subjectKeyIdentifier',
        },
      ]);
      cert.sign(keys.privateKey, md.sha256.create());
    }

    // convert a Forge certificate to PEM
    const pem = pki.certificateToPem(cert);
    const pemPrivateKey = pki.privateKeyToPem(keys.privateKey);

    return {
      publicKey: pem,
      privateKey: pemPrivateKey,
    };
  }

  #createHttpsCertificate(
    hostname: string,
    rootCA: { privateKey: PKI.PrivateKey; certificate: PKI.Certificate },
  ): { privateKey: string; publicKey: string } {
    // generate a keypair and create an X.509v3 certificate
    const keys = pki.rsa.generateKeyPair(2048);
    const cert = pki.createCertificate();

    cert.publicKey = keys.publicKey;
    cert.privateKey = keys.privateKey;
    cert.serialNumber = '01' + randomBytes(19).toString('hex');
    cert.validity.notBefore = new Date();
    cert.validity.notAfter = new Date();
    cert.validity.notAfter.setFullYear(cert.validity.notBefore.getFullYear() + 10);
    const attrs = [
      {
        name: 'commonName',
        value: hostname,
      },
      ...CERTIFICATE_ATTRS,
    ];
    cert.setSubject(attrs);

    cert.setIssuer(rootCA.certificate.subject.attributes);

    const subjectAltName = isIp(hostname)
      ? [
          {
            type: 7, // 7 is IP type
            ip: hostname,
          },
        ]
      : [
          {
            type: 2, // 2 is DNS type
            value: hostname,
          },
        ];

    cert.setExtensions([
      {
        name: 'basicConstraints',
        cA: false,
      },
      {
        name: 'keyUsage',
        digitalSignature: true,
        keyEncipherment: true,
      },
      {
        name: 'extKeyUsage',
        serverAuth: true,
      },
      {
        name: 'subjectAltName',
        altNames: subjectAltName,
      },
      {
        name: 'authorityKeyIdentifier',
        // authorityCertIssuer: true,
        // serialNumber: rootCA.certificate.serialNumber,
        keyIdentifier: rootCA.certificate.generateSubjectKeyIdentifier().getBytes(),
      },
    ]);
    cert.sign(rootCA.privateKey, md.sha256.create());

    // convert a Forge certificate to PEM
    const pem = pki.certificateToPem(cert);
    const pemPrivateKey = pki.privateKeyToPem(keys.privateKey);

    return {
      publicKey: pem,
      privateKey: pemPrivateKey,
    };
  }

  /**
   * Generate a private key (rootCA.key) and a X.509 root certificate (rootCA.pem) using node-forge
   * The certificate will have a validity of 10 years.
   * @param path The path were the certificate will be generated
   */
  async generateCertificate(): Promise<void> {
    const rootCAPem = join(this.config.certificatePath, 'rootCA.pem');
    const rootCAKey = join(this.config.certificatePath, 'rootCA.key');

    if (!(await isExists(rootCAPem)) || !(await isExists(rootCAKey))) {
      this.#logger.log('Generating the server authority certificate...');
      const keys = this.#createCertificate('woodstock.shadoware.org', false);

      await mkdir(this.config.certificatePath, { recursive: true });
      await writeFile(rootCAPem, keys.publicKey, 'utf-8');
      await writeFile(rootCAKey, keys.privateKey, 'utf-8');
    }
  }

  async generateHttpsCertificate(): Promise<void> {
    const rootCAPem = join(this.config.certificatePath, 'rootCA.pem');
    const rootCAKey = join(this.config.certificatePath, 'rootCA.key');

    const httpsPem = join(this.config.certificatePath, 'https.pem');
    const httpsKey = join(this.config.certificatePath, 'https.key');

    if (!(await isExists(httpsPem)) || !(await isExists(httpsKey))) {
      this.#logger.log('Generating the https certificate...');

      const rootCA = {
        privateKey: pki.privateKeyFromPem(await readFile(rootCAKey, 'utf-8')),
        certificate: pki.certificateFromPem(await readFile(rootCAPem, 'utf-8')),
      };
      // Generate an HTTPS certificate (not key certificate)
      const keys = this.#createHttpsCertificate(this.config.clientApiHostname, rootCA);

      await mkdir(this.config.certificatePath, { recursive: true });
      await writeFile(httpsPem, keys.publicKey, 'utf-8');
      await writeFile(httpsKey, keys.privateKey, 'utf-8');
    }
  }

  /**
   * Generate the authority certificate for the client (that is a server)
   * The certificate will have a validity of 10 years.
   */
  async #generateClientAuthorityCertificate(host: string): Promise<void> {
    const clientCAPem = join(this.config.certificatePath, host + '_ca.pem');
    const clientCAKey = join(this.config.certificatePath, host + '_ca.key');

    if (!(await isExists(clientCAPem)) || !(await isExists(clientCAKey))) {
      this.#logger.log('Generating the client authority certificate...');
      const keys = this.#createCertificate(host + '.woodstock.shadoware.org', true);

      await mkdir(this.config.certificatePath, { recursive: true });
      await writeFile(clientCAPem, keys.publicKey, 'utf-8');
      await writeFile(clientCAKey, keys.privateKey, 'utf-8');
    }
  }

  async #generateHostServerCertificate(host: string): Promise<void> {
    const rootCAPem = join(this.config.certificatePath, 'rootCA.pem');
    const rootCAKey = join(this.config.certificatePath, 'rootCA.key');

    const hostKey = join(this.config.certificatePath, host + '_client.key');
    const hostCert = join(this.config.certificatePath, host + '_client.pem');

    if (!(await isExists(hostKey)) || !(await isExists(hostCert))) {
      this.#logger.log(`Generating server host ${host} certificate...`);

      const rootCA = {
        privateKey: pki.privateKeyFromPem(await readFile(rootCAKey, 'utf-8')),
        certificate: pki.certificateFromPem(await readFile(rootCAPem, 'utf-8')),
      };

      const keys = this.#createCertificate(host, false, rootCA);

      await writeFile(hostCert, keys.publicKey, 'utf-8');
      await writeFile(hostKey, keys.privateKey, 'utf-8');
    }
  }

  async #generateHostClientCertificate(host: string): Promise<void> {
    const clientCAPem = join(this.config.certificatePath, host + '_ca.pem');
    const clientCAKey = join(this.config.certificatePath, host + '_ca.key');

    const hostKey = join(this.config.certificatePath, host + '_server.key');
    const hostCert = join(this.config.certificatePath, host + '_server.pem');

    if (!(await isExists(hostKey)) || !(await isExists(hostCert))) {
      this.#logger.log(`Generating server host ${host} certificate...`);

      const rootCA = {
        privateKey: pki.privateKeyFromPem(await readFile(clientCAKey, 'utf-8')),
        certificate: pki.certificateFromPem(await readFile(clientCAPem, 'utf-8')),
      };

      const keys = this.#createCertificate(host, true, rootCA);

      await writeFile(hostCert, keys.publicKey, 'utf-8');
      await writeFile(hostKey, keys.privateKey, 'utf-8');
    }
  }

  async #generateHostHttpsCertificate(host: string): Promise<void> {
    const rootCAPem = join(this.config.certificatePath, 'rootCA.pem');
    const rootCAKey = join(this.config.certificatePath, 'rootCA.key');

    const hostKey = join(this.config.certificatePath, host + '_https.key');
    const hostCert = join(this.config.certificatePath, host + '_https.pem');

    if (!(await isExists(hostKey)) || !(await isExists(hostCert))) {
      this.#logger.log(`Generating https host ${host} certificate...`);

      const rootCA = {
        privateKey: pki.privateKeyFromPem(await readFile(rootCAKey, 'utf-8')),
        certificate: pki.certificateFromPem(await readFile(rootCAPem, 'utf-8')),
      };

      const keys = this.#createCertificate(host, false, rootCA);

      await writeFile(hostCert, keys.publicKey, 'utf-8');
      await writeFile(hostKey, keys.privateKey, 'utf-8');
    }
  }

  async generateHostCertificate(host: string): Promise<void> {
    await this.#generateHostServerCertificate(host);

    await this.#generateClientAuthorityCertificate(host);
    await this.#generateHostClientCertificate(host);
    await this.#generateHostHttpsCertificate(host);
  }
}
