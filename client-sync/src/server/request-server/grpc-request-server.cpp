#ifdef WITH_PROTOCOL_GRPC
#include "grpc-request-server.h"
#include "grpc-request-server-exception.h"

#include <QDebug>
#include <QFile>
#include <grpc++/grpc++.h>
#include <manifest/manifest_wrapper.h>

#include <utils/sha256.h>
#include <utils/readfile.h>

using grpc::ClientContext;
using grpc::Status;

/* GrpcBackup */

class GrpcBackup : public Backup
{
public:
    GrpcBackup(woodstock::WoodstockClientService::Stub *stub) : m_stub(stub)
    {
        writer = m_stub->UpdateFileManifest(&m_context, &m_reply);
    };

    virtual void updateFileManifest(const Common::JournalEntry &entry) override
    {
        woodstock::FileManifestJournalEntry protobuf;
        entry.toProtobuf(&protobuf);
        writer->Write(protobuf);
    }

    void close()
    {
        if (!writer->WritesDone())
        {
            throw GrpcRequestServerException("Writes Done");
        }
        if (m_reply.code() == woodstock::StatusCode::Failed)
        {
            throw GrpcRequestServerException("Wrong status code");
        }
    }

private:
    ClientContext m_context;
    woodstock::UpdateManifestReply m_reply;
    woodstock::WoodstockClientService::Stub *m_stub;
    std::unique_ptr<grpc::ClientWriter<woodstock::FileManifestJournalEntry>> writer;
};

/* GrpcRequestServer */

GrpcRequestServer::GrpcRequestServer(const QString &host)
{
    grpc::SslCredentialsOptions sslOpts;
    sslOpts.pem_root_certs = readKeycert("../certs/server.crt");

    auto sslCreds = grpc::SslCredentials(sslOpts);

    grpc::ChannelArguments args;
    args.SetCompressionAlgorithm(GRPC_COMPRESS_GZIP);

    auto grpc = grpc::CreateCustomChannel((host + ":3657").toStdString(), sslCreds, args);
    m_stub = woodstock::WoodstockClientService::NewStub(grpc);
}

std::unique_ptr<GrpcRequestServer>
GrpcRequestServer::create(const QString &host)
{
    return std::unique_ptr<GrpcRequestServer>(new GrpcRequestServer(host));
}

PrepareResult GrpcRequestServer::prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid)
{
    ClientContext context;
    woodstock::PrepareBackupRequest request;
    woodstock::PrepareBackupReply reply;

    configuration.toProtobuf(request.mutable_configuration());

    auto response = m_stub->PrepareBackup(&context, request, &reply);
    if (!response.ok())
    {
        throw GrpcRequestServerException("Preparation of backup has been cancelled by client");
    }

    auto statusCode = reply.code();
    auto needRefreshCache = reply.needrefreshcache();

    if (statusCode == woodstock::StatusCode::Failed)
    {
        throw GrpcRequestServerException("Preparation of backup has been failed");
    }

    return {needRefreshCache};
}

void GrpcRequestServer::refreshCache(ManifestWrapper *wrapper)
{
    ClientContext context;
    woodstock::RefreshCacheReply reply;
    woodstock::FileManifest manifest;

    auto writer = m_stub->RefreshCache(&context, &reply);
    wrapper->readAllMessages<woodstock::FileManifest>([&](const woodstock::FileManifest &fileManifest, const std::streampos &pos) {
        if (!writer->Write(manifest, grpc::WriteOptions().set_buffer_hint()))
        {
            throw GrpcRequestServerException("Can't send refresh cache to the client: Write ko");
        }
    });

    if (!writer->WritesDone())
    {
        throw GrpcRequestServerException("Can't send refresh cache to the client: WritesDone ko");
    }
}

std::unique_ptr<Backup> GrpcRequestServer::updateFileManifest()
{
    return NULL;
}

void GrpcRequestServer::launchBackup(qint32 backupNumber, std::function<void(const Common::JournalEntry &)> process)
{
    ClientContext context;
    woodstock::LaunchBackupRequest request;
    woodstock::FileManifestJournalEntry journalEntry;

    request.set_backupnumber(backupNumber);

    auto response = m_stub->LaunchBackup(&context, request);

    while (auto cont = response->Read(&journalEntry))
    {
        auto entry = Common::JournalEntry::fromProtobuf(journalEntry);
        process(entry);
    }
}

qint64 GrpcRequestServer::getChunk(const QString filename, qint64 pos, qint64 size, PoolChunkWrapper *device)
{
    ClientContext context;
    woodstock::GetChunkRequest request;
    woodstock::FileChunk chunk;

    qint64 totalRead = 0;

    request.set_filename(filename.toStdString());
    request.set_position(pos);
    request.set_size(size);
    request.set_sha256(device->sha256().toStdString());

    auto response = m_stub->GetChunk(&context, request);

    while (auto cont = response->Read(&chunk))
    {
        device->getDevice()->write(chunk.data().c_str(), chunk.data().length());
        totalRead += chunk.data().length();
    }

    return totalRead;
}
#endif