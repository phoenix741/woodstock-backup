import { Inject, Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { readFile, writeFile } from 'fs/promises';
import * as mkdirp from 'mkdirp';
import type { pki as PKI } from 'node-forge';
import { md, pki } from 'node-forge';
import { join } from 'path';
import { ApplicationConfigService } from '../config';
import { WorkerType, WORKER_TYPE } from '../shared';
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

const CERTIFICATE_EXTENSIONS = [
  {
    name: 'basicConstraints',
    cA: true,
  },
  {
    name: 'keyUsage',
    keyCertSign: true,
    digitalSignature: true,
    nonRepudiation: true,
    keyEncipherment: true,
    dataEncipherment: true,
  },
  {
    name: 'extKeyUsage',
    serverAuth: true,
    clientAuth: true,
    codeSigning: true,
    emailProtection: true,
    timeStamping: true,
  },
  {
    name: 'nsCertType',
    client: true,
    server: true,
    email: true,
    objsign: true,
    sslCA: true,
    emailCA: true,
    objCA: true,
  },
  {
    name: 'subjectKeyIdentifier',
  },
];

@Injectable()
export class CertificateService implements OnModuleInit {
  #logger = new Logger(CertificateService.name);

  constructor(@Inject(WORKER_TYPE) private workerType: WorkerType, private config: ApplicationConfigService) {}

  async onModuleInit() {
    if (this.workerType === WorkerType.api) {
      await this.generateCertificate();
    }
  }

  #createCertificate(
    host: string,
    rootCA?: { privateKey: PKI.PrivateKey; certificate: PKI.Certificate },
  ): { privateKey: string; publicKey: string } {
    // generate a keypair and create an X.509v3 certificate
    const keys = pki.rsa.generateKeyPair(2048);
    const cert = pki.createCertificate();

    cert.publicKey = keys.publicKey;
    cert.serialNumber = '01';
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
    cert.setIssuer(attrs);
    cert.setExtensions(CERTIFICATE_EXTENSIONS);

    if (rootCA) {
      cert.setIssuer(rootCA.certificate.subject.attributes);
      cert.sign(rootCA.privateKey, md.sha256.create());
    } else {
      cert.sign(keys.privateKey);
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
      this.#logger.log('Generating root certificate...');
      const keys = this.#createCertificate('woodstock.shadoware.org');

      await mkdirp(this.config.certificatePath);
      await writeFile(rootCAPem, keys.publicKey, 'utf-8');
      await writeFile(rootCAKey, keys.privateKey, 'utf-8');
    }
  }

  async generateHostCertificate(host: string): Promise<void> {
    const rootCAPem = join(this.config.certificatePath, 'rootCA.pem');
    const rootCAKey = join(this.config.certificatePath, 'rootCA.key');

    const hostKey = join(this.config.certificatePath, host + '.key');
    const hostCert = join(this.config.certificatePath, host + '.pem');

    if (!(await isExists(hostKey)) || !(await isExists(hostCert))) {
      this.#logger.log(`Generating host ${host} certificate...`);

      const rootCA = {
        privateKey: pki.privateKeyFromPem(await readFile(rootCAKey, 'utf-8')),
        certificate: pki.certificateFromPem(await readFile(rootCAPem, 'utf-8')),
      };

      const keys = this.#createCertificate(host, rootCA);

      await writeFile(hostCert, keys.publicKey, 'utf-8');
      await writeFile(hostKey, keys.privateKey, 'utf-8');
    }
  }
}
