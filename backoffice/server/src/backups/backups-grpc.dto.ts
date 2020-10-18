import { LoggerService } from '@nestjs/common';
import { ClientGrpcProxy } from '@nestjs/microservices';
import { ChunkInformation, WoodstockClientServiceClient } from '@woodstock/shared';
import { Observable, Subject, Unsubscribable } from 'rxjs';
import { PoolChunkError, PoolChunkInformation } from './pool/pool-chunk.dto';

export class BackupsGrpcContext {
  public sessionId?: string;
  public logger?: LoggerService;
  public subscriptions: Unsubscribable[] = [];
  public chunkInformation?: Subject<ChunkInformation>;
  public chunkResult?: Observable<PoolChunkInformation | PoolChunkError>;

  constructor(public host: string, public ip: string, public currentBackupId: number, public client: ClientGrpcProxy) {}

  get service() {
    return this.client.getClientByServiceName<WoodstockClientServiceClient>('WoodstockClientService');
  }
}
