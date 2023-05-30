import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { EncryptionService } from '@woodstock/shared';
import { ClientConfigService } from '../client.config';
import { v4 as uuidv4 } from 'uuid';
import { Jwt } from 'jsonwebtoken';

export interface JwtPayload {
  authenticated: boolean;
  sessionId: string;
}

@Injectable()
export class AuthService {
  private context = new Set<string>();

  constructor(
    private clientConfig: ClientConfigService,
    private encryptionService: EncryptionService,
    private jwtService: JwtService,
  ) {}

  /**
   * Create a session token that confirm that the client is authenticated.
   * @param token A token from the server signed with RS256 from the server
   * @returns A session token signed with HS256
   */
  async authenticate(token: string) {
    // Check token validity
    await this.encryptionService.verifyAuthentificationToken(
      this.clientConfig.config.hostname,
      token,
      this.clientConfig.config.password,
    );

    const uuid = uuidv4();
    const sessionToken = await this.jwtService.signAsync(
      { sessionId: uuid, authenticated: true } satisfies JwtPayload,
      {
        algorithm: 'HS256',
        issuer: this.clientConfig.config.hostname,
        audience: this.clientConfig.config.hostname,
        subject: uuid,
        expiresIn: this.clientConfig.config.backupTimeout,
      },
    );

    this.context.add(uuid);

    return sessionToken;
  }

  /**
   * Check that the token isn't expired and valid (signed with HS256).
   * The token have a lifetime of 1 hour, we have a tolerance of 5 minutes, on the
   * clock to renew token if expired.
   * @param sessionId The session token
   * @returns The session id
   */
  async checkContext(sessionId: string) {
    const payload = await this.jwtService.verifyAsync<JwtPayload>(sessionId, {
      issuer: this.clientConfig.config.hostname,
      audience: this.clientConfig.config.hostname,
      algorithms: ['HS256'],
    });

    if (!this.context.has(payload.sessionId)) {
      throw new UnauthorizedException('Session not found');
    }

    if (!payload.authenticated) {
      throw new UnauthorizedException('Session not found');
    }

    return payload.sessionId;
  }

  /**
   * Remove the session token from the context.
   */
  async logout(sessionToken: string) {
    const sessionId = await this.checkContext(sessionToken);

    this.context.delete(sessionId);
  }
}
