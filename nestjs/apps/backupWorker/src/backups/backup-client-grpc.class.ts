import { credentials, Metadata } from '@grpc/grpc-js';
import { Injectable, Logger, LoggerService } from '@nestjs/common';
import { ClientGrpcProxy, ClientProxyFactory, Transport } from '@nestjs/microservices';
import {
  AuthenticateReply,
  ChunkInformation,
  ChunkStatus,
  ExecuteCommandReply,
  FileManifestJournalEntry,
  GetChunkReply,
  LaunchBackupReply,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  Share,
  WoodstockClientServiceClient
} from '@woodstock/shared';
import { readFile } from 'fs/promises';
import { asAsyncIterable } from 'ix';
import { AsyncIterableX, from, pipe } from 'ix/asynciterable';
import { filter, map } from 'ix/asynciterable/operators';
import { resolve } from 'path';
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

  createConnection(ip: string, hostname: string, currentBackupId: number): Observable<BackupsGrpcContext> {
    return defer(async () => {
      this.logger.log(`Create connection to ${hostname} (${ip})`);

      const channel_creds = credentials.createSsl(
        await readFile('./certs/rootCA.pem'),
        await readFile('./certs/server.key'),
        await readFile('./certs/server.crt'),
      );
      const client = ClientProxyFactory.create({
        transport: Transport.GRPC,
        options: {
          package: 'woodstock',
          protoPath: resolve('woodstock.proto'),
          url: ip + ':3657',
          credentials: channel_creds,
          channelOptions: {
            'grpc.ssl_target_name_override': 'pc-ulrich.eden.lan',
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

  authenticate(context: BackupsGrpcContext): Promise<AuthenticateReply> {
    return new Promise<AuthenticateReply>((resolve, reject) => {
      context.service.authenticate({ version: 0 }, (err, reply) => {
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
      const writer = context.service.refreshCache(this.getMetadata(context), (err, response) => {
        if (err) {
          reject(err);
        } else {
          resolve(response);
        }
      });
      pipeline(Readable.from(request), writer as unknown as NodeJS.WritableStream).catch((err) => {
        reject(err);
      }); // FIXME: abort
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
      pipe(
        chunkResult,
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
