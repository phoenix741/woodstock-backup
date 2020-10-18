#pragma once

#include <memory>

#include "request-client.h"
#include "woodstock.grpc.pb.h"

using grpc::Server;
using grpc::ServerContext;
using grpc::ServerReader;
using grpc::ServerWriter;
using grpc::Status;

using woodstock::FileChunk;
using woodstock::FileManifest;
using woodstock::FileManifestJournalEntry;
using woodstock::GetChunkRequest;
using woodstock::LaunchBackupRequest;
using woodstock::LogEntry;
using woodstock::PrepareBackupReply;
using woodstock::PrepareBackupRequest;
using woodstock::RefreshCacheReply;
using woodstock::StreamLogRequest;

class GrpcRequestClient : public RequestClient,
                          protected woodstock::WoodstockClientService::Service
{
public:
    GrpcRequestClient(RequestClientImplementation *impl);
    virtual ~GrpcRequestClient();

    virtual void listen() override;
    virtual void stop() override;

    static std::unique_ptr<GrpcRequestClient> create(RequestClientImplementation *impl);

protected:
    virtual Status PrepareBackup(ServerContext *context, const PrepareBackupRequest *request, PrepareBackupReply *response) override;
    virtual Status RefreshCache(ServerContext *context, ServerReader<FileManifest> *reader, RefreshCacheReply *response) override;
    virtual Status LaunchBackup(ServerContext *context, const LaunchBackupRequest *request, ServerWriter<FileManifestJournalEntry> *writer) override;
    virtual Status GetChunk(ServerContext *context, const GetChunkRequest *request, ServerWriter<FileChunk> *writer) override;
    virtual Status StreamLog(ServerContext *context, const StreamLogRequest *request, ServerWriter<LogEntry> *writer) override;

private:
    std::unique_ptr<Server> m_server;
};
