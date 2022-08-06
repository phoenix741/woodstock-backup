import { credentials, Metadata } from '@grpc/grpc-js';
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
import { filter, map } from 'ix/asynciterable/operators';
import { join, resolve } from 'path';
import { defer, Observable } from 'rxjs';
import { Readable } from 'stream';
import { pipeline } from 'stream/promises';
import { BackupClientContext, BackupClientInterface } from './backup-client.interface';
import { LaunchBackupError } from './backup.error';

export class BackupsGrpcContext implements BackupClientContext {
  public sessionId?: string;
  public logger?: LoggerService;
  public abortable: AbortController[] = [];

  constructor(public host: string, public ip: string, public currentBackupId: number, public client: ClientGrpcProxy) {}

  get service() {
    return this.client.getClientByServiceName<WoodstockClientServiceClient>('WoodstockClientService');
  }
}

@Injectable()
export class BackupClientGrpc implements BackupClientInterface {
  private readonly logger = new Logger(BackupClientGrpc.name);

  constructor(private config: ApplicationConfigService, private encryptionService: EncryptionService) {}

  createConnection(ip: string, hostname: string, currentBackupId: number): Observable<BackupsGrpcContext> {
    return defer(async () => {
      this.logger.log(`Create connection to ${hostname} (${ip})`);

      const channel_creds = credentials.createSsl(
        await readFile(join(this.config.certificatePath, 'rootCA.pem')),
        await readFile(join(this.config.certificatePath, `${hostname}.key`)),
        await readFile(join(this.config.certificatePath, `${hostname}.pem`)),
      );
      const client = ClientProxyFactory.create({
        transport: Transport.GRPC,
        options: {
          package: 'woodstock',
          protoPath: resolve('woodstock.proto'),
          url: ip + ':3657',
          credentials: channel_creds,
          channelOptions: {
            'grpc.ssl_target_name_override': hostname,
            'grpc.enable_channelz': 0,
            'grpc.default_compression_algorithm': 2,
            'grpc.default_compression_level': 2,
          },
        },
      });

      return new BackupsGrpcContext(hostname, ip, currentBackupId, client);
    });
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
      context.service.authenticate({ version: 0, token }, (err, reply) => {
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
    return context.service
      .streamLog({}, this.getMetadata(context))
      .pipe(asAsyncIterable<LogEntry>({ objectMode: true }));
  }

  async executeCommand(context: BackupsGrpcContext, command: string): Promise<void> {
    const executeCommand = new Promise<ExecuteCommandReply>((resolve, reject) => {
      context.service.executeCommand({ command }, this.getMetadata(context), (err, reply) => {
        if (err) {
          reject(err);
          return;
        }
        resolve(reply);
      });
    });

    const reply = await executeCommand;

    reply?.stderr && context.logger?.error(reply.stderr);
    reply?.stdout && context.logger?.error(reply.stdout);

    if (!reply || reply.code) {
      context.logger?.log(`The command "${command}" has been executed with error: ${reply?.code}`, 'executeCommand');
      throw new LaunchBackupError(reply.stderr || `Can\' execute the command ${command}`);
    } else {
      context.logger?.log(`The command "${command}" has been executed successfully.`, 'executeCommand');
    }
  }

  refreshCache(context: BackupsGrpcContext, request: AsyncIterable<RefreshCacheRequest>): Promise<RefreshCacheReply> {
    return new Promise((resolve, reject) => {
      const abortController = new AbortController();
      const reader = Readable.from(request);
      const writer = context.service.refreshCache(this.getMetadata(context), (err, response) => {
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
    const grpclaunchBackup = context.service.launchBackup({ share: backupShare }, this.getMetadata(context));
    return from<LaunchBackupReply>(grpclaunchBackup).pipe(
      map(({ entry }) => entry),
      filter((entry): entry is FileManifestJournalEntry => !!entry),
    );
  }

  copyChunk(context: BackupsGrpcContext, chunk: ChunkInformation): Readable {
    const chunkResult = context.service.getChunk({ chunk }, this.getMetadata(context));
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
    context.client.close();
  }
}
