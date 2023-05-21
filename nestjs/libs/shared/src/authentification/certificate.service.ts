import { Inject, Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ApplicationConfigService, isExists, WorkerType, WORKER_TYPE } from '@woodstock/core';
import { randomBytes } from 'crypto';
import { mkdir, readFile, writeFile } from 'fs/promises';
import type { pki as PKI } from 'node-forge';
import { md, pki } from 'node-forge';
import { join } from 'path';

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

@Injectable()
export class CertificateService implements OnModuleInit {
  #logger = new Logger(CertificateService.name);

  constructor(
    @Inject(WORKER_TYPE) private workerType: WorkerType,
    private config: ApplicationConfigService,
  ) {}

  async onModuleInit() {
    if (this.workerType === WorkerType.api) {
      await this.generateCertificate();
    }
  }

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
              altNames: [
                {
                  type: 2, // 2 is DNS type
                  value: host,
                },
              ],
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

  async generateHostCertificate(host: string): Promise<void> {
    await this.#generateHostServerCertificate(host);

    await this.#generateClientAuthorityCertificate(host);
    await this.#generateHostClientCertificate(host);
  }
}
