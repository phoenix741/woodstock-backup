import { credentials, Metadata } from '@grpc/grpc-js';
import { ConnectivityState } from '@grpc/grpc-js/build/src/connectivity-state';
import { Injectable, Logger, LoggerService, UnauthorizedException } from '@nestjs/common';
import { ClientGrpcProxy, ClientProxyFactory, Transport } from '@nestjs/microservices';
import {
  ApplicationConfigService,
  AuthenticateReply,
  ChunkInformation,
  ChunkStatus,
  EncryptionService,
  ExecuteCommandReply,
  FileManifestJournalEntry,
  GetChunkReply,
  LaunchBackupReply,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  StatusCode,
  WoodstockClientServiceClient,
} from '@woodstock/shared';
import { readFile } from 'fs/promises';
import { asAsyncIterable } from 'ix';
import { AsyncIterableX, from } from 'ix/asynciterable';
import { catchError, filter, map } from 'ix/asynciterable/operators';
import { join, resolve } from 'path';
import { Readable } from 'stream';
import { pipeline } from 'stream/promises';
import { BackupClientContext, BackupClientInterface } from './backup-client.interface.js';
import { LaunchBackupError } from './backup.error.js';

export class BackupsGrpcContext implements BackupClientContext {
  sessionId?: string;
  logger?: LoggerService;
  abortable: AbortController[] = [];
  client?: ClientGrpcProxy;

  constructor(public host: string, public ip: string | undefined, public currentBackupId: number) {}

  get service() {
    return this.client?.getClientByServiceName<WoodstockClientServiceClient>('WoodstockClientService');
  }
}

@Injectable()
export class BackupClientGrpc implements BackupClientInterface {
  private readonly logger = new Logger(BackupClientGrpc.name);

  constructor(private config: ApplicationConfigService, private encryptionService: EncryptionService) {}

  checkConnection(context: BackupsGrpcContext): WoodstockClientServiceClient {
    if (!context.service) {
      throw new LaunchBackupError(`No connection to host ${context.host}`);
    }

    const status = context.service?.getChannel().getConnectivityState(true);
    if ([ConnectivityState.SHUTDOWN, ConnectivityState.TRANSIENT_FAILURE].includes(status)) {
      throw new LaunchBackupError(`Connection to host ${context.host} is down`);
    }

    return context.service;
  }

  createContext(ip: string | undefined, hostname: string, currentBackupId: number): BackupsGrpcContext {
    this.logger.log(`Create context to ${hostname} (${ip})`);

    return new BackupsGrpcContext(hostname, ip, currentBackupId);
  }

  async createConnection(context: BackupsGrpcContext): Promise<void> {
    this.logger.log(`Create connection to ${context.host} (${context.ip})`);
    if (!context.ip) {
      throw new Error(`No ip for ${context.host}`);
    }

    const channel_creds = credentials.createSsl(
      await readFile(join(this.config.certificatePath, 'rootCA.pem')),
      await readFile(join(this.config.certificatePath, `${context.host}.key`)),
      await readFile(join(this.config.certificatePath, `${context.host}.pem`)),
    );
    const client = ClientProxyFactory.create({
      transport: Transport.GRPC,
      options: {
        package: 'woodstock',
        protoPath: resolve('woodstock.proto'),
        url: context.ip + ':3657',
        credentials: channel_creds,
        channelOptions: {
          'grpc.ssl_target_name_override': context.host,
          'grpc.enable_channelz': 0,
          'grpc.default_compression_algorithm': 2,
          'grpc.default_compression_level': 2,
        },
      },
    });

    context.client = client;
  }

  private getMetadata(context: BackupsGrpcContext) {
    if (!context.sessionId) {
      throw new LaunchBackupError("Can't find the sessionId");
    }

    const metadata = new Metadata();
    metadata.add('X-Session-Id', context.sessionId);
    return metadata;
  }

  async authenticate(context: BackupsGrpcContext, password: string): Promise<AuthenticateReply> {
    const token = await this.encryptionService.createAuthentificationToken(context.host, password);
    if (!token) {
      throw new UnauthorizedException("The token for the backup can't be generated");
    }

    return await new Promise<AuthenticateReply>((resolve, reject) => {
      const service = this.checkConnection(context);

      service.authenticate({ version: 0, token }, (err, reply) => {
        if (err) {
          reject(err);
          return;
        }

        context.sessionId = reply.sessionId;
        resolve(reply);
      });
    });
  }

  streamLog(context: BackupsGrpcContext): AsyncIterableX<LogEntry> {
    if (!context.service) {
      throw new LaunchBackupError(`No connection tho host ${context.host}`);
    }

    return context.service
      .streamLog({}, this.getMetadata(context))
      .on('error', (err) => {
        context.logger?.error(`Can't get the stream from the client: ${err.message}`);
      })
      .pipe(asAsyncIterable<LogEntry>({ objectMode: true }));
  }

  async executeCommand(context: BackupsGrpcContext, command: string): Promise<ExecuteCommandReply> {
    const executeCommand = new Promise<ExecuteCommandReply>((resolve, reject) => {
      const service = this.checkConnection(context);

      service.executeCommand({ command }, this.getMetadata(context), (err, reply) => {
        if (err) {
          reject(err);
          return;
        }
        resolve(reply);
      });
    });

    const reply = await executeCommand;

    reply?.stderr && context.logger?.error(reply.stderr);
    reply?.stdout && context.logger?.log(reply.stdout);

    if (!reply || reply.code) {
      context.logger?.log(`The command "${command}" has been executed with error: ${reply?.code}`, 'executeCommand');
      throw new LaunchBackupError(reply.stderr || `Can\' execute the command ${command}`);
    } else {
      context.logger?.log(`The command "${command}" has been executed successfully.`, 'executeCommand');
    }

    return reply;
  }

  refreshCache(context: BackupsGrpcContext, request: AsyncIterable<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    return new Promise((resolve, reject) => {
      const service = this.checkConnection(context);

      const abortController = new AbortController();
      const reader = Readable.from(request);
      const writer = service.refreshCache(this.getMetadata(context), (err, response) => {
        this.logger.log(
          `Recieve response from refresh cache : ${err?.message} : ${response?.code} ${response?.message || ''}`,
        );

        if (err || response?.code === StatusCode.Failed) {
          abortController.abort();
          reject(err || new Error(response.code + ' ' + response.message));
        } else {
          resolve(response);
        }
      });
      pipeline(reader, writer as unknown as NodeJS.WritableStream, { signal: abortController.signal }).catch((err) => {
        reject(err);
      });
    });
  }

  downloadFileList(context: BackupsGrpcContext, backupShare: Share): AsyncIterableX<FileManifestJournalEntry> {
    const service = this.checkConnection(context);

    const grpclaunchBackup = service.launchBackup({ share: backupShare }, this.getMetadata(context));
    return from<LaunchBackupReply>(grpclaunchBackup).pipe(
      map(({ entry }) => entry),
      filter((entry): entry is FileManifestJournalEntry => !!entry),
      map((entry) => {
        // TODO: Better Xfer log
        context.logger?.log(`${entry.type} ${entry.manifest?.path.toString('utf-8')}`);
        return entry;
      }),
    );
  }

  copyChunk(context: BackupsGrpcContext, chunk: ChunkInformation): Readable {
    const service = this.checkConnection(context);

    const chunkResult = service.getChunk({ chunk }, this.getMetadata(context));
    chunkResult.on('error', () => {
      context.logger?.error(`Can't read the chunk ${chunk.sha256}`);
    });
    return Readable.from(
      from(chunkResult).pipe(
        map<GetChunkReply, GetChunkReply>((pieceOfChunk) => {
          if (pieceOfChunk.status === ChunkStatus.ERROR) {
            throw new Error(`Can't read the chunk ${chunk.sha256}`);
          }
          return pieceOfChunk;
        }),
        filter<GetChunkReply>((pieceOfChunk) => pieceOfChunk.status === ChunkStatus.DATA),
        map<GetChunkReply, Buffer | undefined>((pieceOfChunk) => pieceOfChunk.data?.data),
        filter<Buffer>((buffer): buffer is Buffer => !!buffer),
      ),
    );
  }

  close(context: BackupsGrpcContext): void {
    context.client?.close();
  }
}
