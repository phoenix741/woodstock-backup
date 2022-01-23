#ifdef WITH_PROTOCOL_GRPC
#include "./grpc-request-client.h"
#include "./grpc-request-client-exception.h"

#include <QUuid>
#include <QDebug>
#include <QFile>
#include <grpc++/grpc++.h>
#include <devices/sha256device.h>

#include <utils/readfile.h>

using grpc::ServerBuilder;

constexpr const std::size_t GRPC_BUFFER_SIZE{1 << 17};

GrpcRequestClient::GrpcRequestClient(RequestClientImplementation *impl) : RequestClient(impl)
{
}

GrpcRequestClient::~GrpcRequestClient()
{
}

std::unique_ptr<GrpcRequestClient> GrpcRequestClient::create(RequestClientImplementation *impl)
{
    return std::unique_ptr<GrpcRequestClient>(new GrpcRequestClient(impl));
}

void GrpcRequestClient::listen()
{
    std::string server_address("0.0.0.0:3657");

    std::string servercert = readKeycert("../certs/server.crt");
    std::string serverkey = readKeycert("../certs/server.key");

    grpc::SslServerCredentialsOptions::PemKeyCertPair pkcp;
    pkcp.private_key = serverkey;
    pkcp.cert_chain = servercert;

    grpc::SslServerCredentialsOptions sslOpts;
    sslOpts.pem_root_certs = "";
    sslOpts.pem_key_cert_pairs.push_back(pkcp);

    std::shared_ptr<grpc::ServerCredentials>
        creds = grpc::SslServerCredentials(sslOpts);

    ServerBuilder builder;
    builder.SetDefaultCompressionAlgorithm(GRPC_COMPRESS_GZIP);

    builder.AddListeningPort(server_address, creds);

    builder.RegisterService(this);

    m_server = builder.BuildAndStart();
    std::cout << "Server listening on " << server_address << std::endl;
}

void GrpcRequestClient::stop()
{
    m_server->Shutdown();
    m_server.reset();
}

Status GrpcRequestClient::PrepareBackup(ServerContext *context, const PrepareBackupRequest *request, PrepareBackupReply *response)
{
    try
    {
        auto grpcConfiguration = request->configuration();
        auto configuration = Common::Configuration::fromProtobuf(grpcConfiguration);
        auto result = this->m_impl->prepareBackup(configuration, request->lastbackupnumber(), request->newbackupnumber());

        response->set_code(woodstock::StatusCode::Ok);
        response->set_needrefreshcache(result.needRefreshCache);

        return Status::OK;
    }
    catch (...)
    {
        // Log Error
        std::cout << "Error while prepare backup" << std::endl;

        response->set_code(woodstock::StatusCode::Failed);
        return Status::OK;
    }
}

Status GrpcRequestClient::RefreshCache(ServerContext *context, ServerReader<FileManifest> *reader, RefreshCacheReply *response)
{
    try
    {
        auto cacheClient = this->m_impl->refreshCache();

        woodstock::FileManifest manifest;

        while (auto cont = reader->Read(&manifest))
        {
            auto commonManifest = Common::FileManifest::fromProtobuf(manifest);
            cacheClient->addFileManifest(commonManifest);
        }

        cacheClient->close();

        response->set_code(woodstock::StatusCode::Ok);
        return Status::OK;
    }
    catch (...)
    {
        // Log Error
        std::cout << "Error while refresh cache" << std::endl;

        response->set_code(woodstock::StatusCode::Failed);
        return Status::OK;
    }
}

Status GrpcRequestClient::LaunchBackup(ServerContext *context, const LaunchBackupRequest *request, ServerWriter<FileManifestJournalEntry> *writer)
{
    try
    {
        m_impl->launchBackup(request->backupnumber(), [writer](const Common::JournalEntry &entry) {
            woodstock::FileManifestJournalEntry pbEntry;
            entry.toProtobuf(&pbEntry);

            return writer->Write(pbEntry);
        });

        woodstock::FileManifestJournalEntry closedEntry;
        closedEntry.set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_CLOSE);
        writer->WriteLast(closedEntry, grpc::WriteOptions());

        return Status::OK;
    }
    catch (...)
    {
        // Log Error
        std::cout << "Error while prepare backup" << std::endl;

        return Status::CANCELLED;
    }
}

Status GrpcRequestClient::GetChunk(ServerContext *context, const GetChunkRequest *request, ServerWriter<FileChunk> *writer)
{
    try
    {
        auto filename = QString::fromStdString(request->filename());
        auto sha256 = QByteArray::fromStdString(request->sha256());
        auto chunkDevice = m_impl->getChunk(filename, request->position());
        auto sha256Device = std::unique_ptr<Sha256Device>(new Sha256Device(chunkDevice.release()));
        if (!sha256Device->open(QIODevice::ReadOnly))
        {
            throw std::runtime_error("Can't open the file readonly"); // FIXME, should be an cancel ?
        }

        char buffer[GRPC_BUFFER_SIZE];
        woodstock::FileChunk fileChunkRequest;
        qint64 totalRead = 0;

        for (auto chunkPos = 0; (chunkPos < request->size()) && !sha256Device->atEnd(); chunkPos += GRPC_BUFFER_SIZE)
        {
            auto readSize = qMin(GRPC_BUFFER_SIZE, request->size() - chunkPos);
            auto gcount = sha256Device->read(buffer, readSize);
            if (gcount > 0)
            {
                fileChunkRequest.set_data(buffer, gcount);
                if (!writer->Write(fileChunkRequest))
                {
                    // Stream is closed (cancel)
                    qDebug() << "Stream is closed for " << sha256.toHex();
                    break;
                }
            }

            totalRead += gcount;
        }

        if (sha256 != sha256Device->getHash())
        {
            qDebug() << "Chunk " << sha256.toHex() << " of file " << filename << " has vanished"; // FIXME: Log
            return Status::CANCELLED;                                                             // Server should repeat or store the chunk in another space
        }

        if (request->size() < totalRead)
        {
            throw GrpcRequestClientException("Size of readed data different of real date to read");
        }

        sha256Device->close();
        return Status::OK;
    }
    catch (const std::runtime_error &e)
    {
        // Log Error
        std::cout << "Error while getting chunk" << e.what() << std::endl;

        return Status::CANCELLED;
    }
    catch (...)
    {
        // Log Error
        std::cout << "Error while getting chunk" << std::endl;

        return Status::CANCELLED;
    }
}

Status GrpcRequestClient::StreamLog(ServerContext *context, const StreamLogRequest *request, ServerWriter<LogEntry> *writer)
{
    return Status::CANCELLED;
}
#endif