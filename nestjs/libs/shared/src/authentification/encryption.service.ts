import { Inject, Injectable, Logger, OnModuleInit, UnauthorizedException } from '@nestjs/common';
import { createHash, randomBytes } from 'crypto';
import { readFile, writeFile } from 'fs/promises';
import type { GetPublicKeyOrSecret, Jwt, JwtPayload, Secret, SignOptions, VerifyOptions } from 'jsonwebtoken';
import { sign, verify } from 'jsonwebtoken';
import * as mkdirp from 'mkdirp';
import { pki } from 'node-forge';
import { join } from 'path';
import { promisify } from 'util';
import { ApplicationConfigService } from '../config';
import { WorkerType, WORKER_TYPE } from '../shared';
import { isExists } from '../utils';

const randomBytesAsync = promisify(randomBytes);

const randomPassword = () => randomBytesAsync(48).then((value) => value.toString('base64'));

export function signAsync(
  payload: string | Buffer | object,
  secretOrPrivateKey: Secret,
  options: SignOptions,
): Promise<string | undefined> {
  return new Promise((resolve, reject) => {
    sign(payload, secretOrPrivateKey, options, (err, token) => {
      if (err) return reject(err);
      else return resolve(token);
    });
  });
}

export function verifyAsync<T = Jwt | JwtPayload | string>(
  token: string,
  secretOrPublicKey: Secret | GetPublicKeyOrSecret,
  options?: VerifyOptions,
): Promise<T | undefined> {
  return new Promise((resolve, reject) => {
    verify(token, secretOrPublicKey, options, (err, token) => {
      if (err) return reject(err);
      else return resolve(token as T);
    });
  });
}

@Injectable()
export class EncryptionService implements OnModuleInit {
  #logger = new Logger(EncryptionService.name);

  constructor(@Inject(WORKER_TYPE) private workerType: WorkerType, private config: ApplicationConfigService) {}

  async onModuleInit() {
    if (this.workerType === WorkerType.api) {
      await this.generateRSAKey();
    }
  }

  async generateRSAKey() {
    const publicKeyFile = join(this.config.certificatePath, 'public_key.pem');
    const privateKeyFile = join(this.config.certificatePath, 'private_key.pem');
    if (!(await isExists(publicKeyFile)) || !(await isExists(privateKeyFile))) {
      this.#logger.log('Generating public and private keys ...');

      const keys = pki.rsa.generateKeyPair(2048);
      const pemPublicKey = pki.publicKeyToPem(keys.publicKey);
      const pemPrivateKey = pki.privateKeyToPem(keys.privateKey);

      await mkdirp(this.config.certificatePath);
      await writeFile(publicKeyFile, pemPublicKey, 'utf-8');
      await writeFile(privateKeyFile, pemPrivateKey, 'utf-8');
    }
  }

  async generatePassword() {
    return await randomPassword();
  }

  async createAuthentificationToken(host: string, password: string) {
    this.#logger.log(`Create the authentification token ${host}`);
    const privateKey = await readFile(join(this.config.certificatePath, 'private_key.pem'), 'utf-8');
    const hash = createHash('sha256').update(password).digest('base64');
    const token = await signAsync({ hash }, privateKey, {
      algorithm: 'RS256',
      expiresIn: '60s',
      issuer: 'woodstock.shadoware.org',
      audience: host,
      subject: host,
    });

    return token;
  }

  async verifyAuthentificationToken(host: string, token: string, password: string) {
    this.#logger.log(`Check the authentification token ${host}`);
    const publicKey = await readFile(join(this.config.clientPath, 'public_key.pem'), 'utf-8');
    const hash = createHash('sha256').update(password).digest('base64');
    try {
      const payload = await verifyAsync<{ hash: string }>(token, publicKey, {
        algorithms: ['RS256'],
        issuer: 'woodstock.shadoware.org',
        audience: host,
        subject: host,
      });

      if (hash !== payload?.hash) {
        throw new UnauthorizedException();
      }

      this.#logger.log('The authentification token is accepted');

      return token;
    } catch (err) {
      this.#logger.error('The authentification token is rejected', err);
      throw err;
    }
  }
}
