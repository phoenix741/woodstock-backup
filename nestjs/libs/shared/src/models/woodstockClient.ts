import {
  CallOptions,
  Client,
  ClientReadableStream,
  ClientUnaryCall,
  ClientWritableStream,
  Metadata,
  ServiceError
} from '@grpc/grpc-js';
import {
  AuthenticateReply,
  AuthenticateRequest,
  ExecuteCommandReply,
  ExecuteCommandRequest,
  GetChunkReply,
  GetChunkRequest,
  LaunchBackupReply,
  LaunchBackupRequest,
  LogEntry,
  RefreshCacheReply,
  RefreshCacheRequest,
  StreamLogRequest
} from '.';

export interface WoodstockClientServiceClient extends Client {
  /** 0. Authenticate */
  authenticate(
    request: AuthenticateRequest,
    callback: (error: ServiceError | null, response: AuthenticateReply) => void,
  ): ClientUnaryCall;
  authenticate(
    request: AuthenticateRequest,
    metadata: Metadata,
    callback: (error: ServiceError | null, response: AuthenticateReply) => void,
  ): ClientUnaryCall;
  authenticate(
    request: AuthenticateRequest,
    metadata: Metadata,
    options: Partial<CallOptions>,
    callback: (error: ServiceError | null, response: AuthenticateReply) => void,
  ): ClientUnaryCall;
  /** 1. Execute a command directly on the client */
  executeCommand(
    request: ExecuteCommandRequest,
    callback: (error: ServiceError | null, response: ExecuteCommandReply) => void,
  ): ClientUnaryCall;
  executeCommand(
    request: ExecuteCommandRequest,
    metadata: Metadata,
    callback: (error: ServiceError | null, response: ExecuteCommandReply) => void,
  ): ClientUnaryCall;
  executeCommand(
    request: ExecuteCommandRequest,
    metadata: Metadata,
    options: Partial<CallOptions>,
    callback: (error: ServiceError | null, response: ExecuteCommandReply) => void,
  ): ClientUnaryCall;
  /** 2. The server send the last cache version to the client */
  refreshCache(
    callback: (error: ServiceError | null, response: RefreshCacheReply) => void,
  ): ClientWritableStream<RefreshCacheRequest>;
  refreshCache(
    metadata: Metadata,
    callback: (error: ServiceError | null, response: RefreshCacheReply) => void,
  ): ClientWritableStream<RefreshCacheRequest>;
  refreshCache(
    options: Partial<CallOptions>,
    callback: (error: ServiceError | null, response: RefreshCacheReply) => void,
  ): ClientWritableStream<RefreshCacheRequest>;
  refreshCache(
    metadata: Metadata,
    options: Partial<CallOptions>,
    callback: (error: ServiceError | null, response: RefreshCacheReply) => void,
  ): ClientWritableStream<RefreshCacheRequest>;
  /**
   * 3. The server launch the backup with a new number. The client will browse the client computer
   *    and will send all new file to the server
   */
  launchBackup(request: LaunchBackupRequest, options?: Partial<CallOptions>): ClientReadableStream<LaunchBackupReply>;
  launchBackup(
    request: LaunchBackupRequest,
    metadata?: Metadata,
    options?: Partial<CallOptions>,
  ): ClientReadableStream<LaunchBackupReply>;
  /**
   * 3. When the server receive a journal entry, the server will compare the chunk with the manifest
   *    and ask all necessary chunk to the client.
   */
  getChunk(request: GetChunkRequest, options?: Partial<CallOptions>): ClientReadableStream<GetChunkReply>;
  getChunk(
    request: GetChunkRequest,
    metadata?: Metadata,
    options?: Partial<CallOptions>,
  ): ClientReadableStream<GetChunkReply>;
  /** In parallele, the server will open a stream for the client to send log file. */
  streamLog(request: StreamLogRequest, options?: Partial<CallOptions>): ClientReadableStream<LogEntry>;
  streamLog(
    request: StreamLogRequest,
    metadata?: Metadata,
    options?: Partial<CallOptions>,
  ): ClientReadableStream<LogEntry>;
}
