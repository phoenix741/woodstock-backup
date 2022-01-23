#ifdef WITH_PROTOCOL_HTTP2
#include "http2-request-client.h"
#include "http2-request-client-exception.h"
#include "protobuf/group_grpc.h"

#include <QByteArray>
#include <QBuffer>
#include <QDebug>

Http2RequestClient::Http2RequestClient(RequestClientImplementation *impl) : RequestClient(impl), server(new http2()), tls(new boost::asio::ssl::context(boost::asio::ssl::context::sslv23)), ec(new boost::system::error_code())
{
}

Http2RequestClient::~Http2RequestClient()
{
}

void Http2RequestClient::listen()
{
    tls->use_private_key_file("../certs/server.key", boost::asio::ssl::context::pem);
    tls->use_certificate_chain_file("../certs/server.crt");

    configure_tls_context_easy(*(ec.get()), *(tls.get()));

    server->handle("/prepare-backup", [this](const request &req, const response &res) {
        try
        {
            std::string *buffer = new std::string();
            req.on_data([this, &res, buffer](const uint8_t *p, std::size_t size) {
                buffer->append((const char *)p, size);
                // qDebug() << QString::fromStdString(*buffer);

                if (size == 0)
                {
                    woodstock::PrepareBackupRequest request;
                    request.ParseFromString(*buffer);
                    // qDebug() << QString::fromStdString(request.DebugString());
                    delete buffer;

                    auto configuration = Common::Configuration::fromProtobuf(request.configuration());
                    auto result = this->m_impl->prepareBackup(configuration, request.lastbackupnumber(), request.newbackupnumber());

                    woodstock::PrepareBackupReply reply;
                    reply.set_code(woodstock::StatusCode::Ok);
                    reply.set_needrefreshcache(result.needRefreshCache);

                    qDebug() << QString::fromStdString(reply.SerializeAsString());
                    res.write_head(200);
                    res.end(reply.SerializeAsString());
                }
            });
        }
        catch (...)
        {
            // Log Error
            std::cout << "Error while prepare backup" << std::endl;

            woodstock::PrepareBackupReply reply;
            reply.set_code(woodstock::StatusCode::Failed);

            res.write_head(200);
            res.end(reply.SerializeAsString());
        }
    });

    server->handle("/refresh-cache", [](const request &req, const response &res) {
        res.write_head(200);
        res.end("hello, world\n");
    });

    server->handle("/launch-backup", [this](const request &req, const response &res) {
        try
        {
            std::string *buffer = new std::string();
            req.on_data([this, &res, buffer](const uint8_t *p, std::size_t size) {
                buffer->append((const char *)p, size);

                if (size == 0)
                {
                    woodstock::LaunchBackupRequest request;
                    request.ParseFromString(*buffer);
                    delete buffer;

                    QBuffer *outputBuffer = new QBuffer();
                    if (!outputBuffer->open(QIODevice::ReadWrite))
                    {
                        throw Http2RequestClientException("Can't open memory buffer");
                    }

                    auto generatorStream = [outputBuffer](uint8_t *buf, size_t len,
                                                          uint32_t *data_flags) -> generator_cb::result_type {
                        // qDebug() << "buffer " << outputBuffer->pos() << outputBuffer->size();
                        ssize_t n;
                        while ((n = outputBuffer->read((char *)buf, len)) == -1 && errno == EINTR)
                            ;

                        if (n == -1)
                        {
                            return NGHTTP2_ERR_TEMPORAL_CALLBACK_FAILURE;
                        }

                        if (n == 0)
                        {
                            *data_flags |= NGHTTP2_DATA_FLAG_EOF;
                            delete outputBuffer;
                        }

                        return n;
                    };

                    m_impl->launchBackup(request.backupnumber(), [outputBuffer](const Common::JournalEntry &entry) {
                        woodstock::FileManifestJournalEntry pbEntry;
                        entry.toProtobuf(&pbEntry);

                        return writeDelimitedTo(outputBuffer, pbEntry);
                    });

                    woodstock::FileManifestJournalEntry closedEntry;
                    closedEntry.set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_CLOSE);
                    writeDelimitedTo(outputBuffer, closedEntry);

                    outputBuffer->seek(0);
                    res.write_head(200);
                    res.end(generatorStream);
                }
            });
        }
        catch (...)
        {
            // Log Error
            std::cout << "Error while prepare backup" << std::endl;

            res.write_head(500);
        }
    });

    server->handle("/get-chunk", [this](const request &req, const response &res) {
        try
        {
            std::string *buffer = new std::string();
            req.on_data([this, &res, buffer](const uint8_t *p, std::size_t size) {
                buffer->append((const char *)p, size);

                if (size == 0)
                {
                    woodstock::GetChunkRequest request;
                    request.ParseFromString(*buffer);
                    delete buffer;

                    auto filename = QString::fromStdString(request.filename());
                    auto sha256 = QByteArray::fromStdString(request.sha256());
                    // qDebug() << "read file " << filename << sha256 << request.position() << request.size();
                    auto chunkDevice = m_impl->getChunk(filename, request.position()).release();
                    chunkDevice->setProperty("size", 0);

                    const auto maxSize = request.size();

                    auto generatorStream = [chunkDevice, maxSize](uint8_t *buf, size_t len,
                                                                  uint32_t *data_flags) -> generator_cb::result_type {
                        ssize_t size = chunkDevice->property("size").toLongLong();
                        auto maxByteToRead = qMin((qlonglong)len, (qlonglong)(maxSize - size));

                        ssize_t n;
                        while ((n = chunkDevice->read((char *)buf, maxByteToRead)) == -1 && errno == EINTR)
                            ;

                        chunkDevice->setProperty("size", (qlonglong)(size + n));

                        if (n == -1)
                        {
                            return NGHTTP2_ERR_TEMPORAL_CALLBACK_FAILURE;
                        }

                        if (n == 0 || (size + maxByteToRead) == maxSize)
                        {
                            *data_flags |= NGHTTP2_DATA_FLAG_EOF;
                            chunkDevice->close();
                            delete chunkDevice;
                        }

                        return n;
                    };

                    res.write_head(200);
                    res.end(generatorStream);
                }
            });
        }
        catch (const std::runtime_error &e)
        {
            // Log Error
            std::cout << "Error while getting chunk" << e.what() << std::endl;

            res.write_head(500);
        }
        catch (...)
        {
            // Log Error
            std::cout << "Error while getting chunk" << std::endl;

            res.write_head(500);
        }
    });

    if (server->listen_and_serve(*(ec.get()), *(tls.get()), "0.0.0.0", "3000", true))
    {
        std::cerr << "error: " << ec->message() << std::endl;
    }
}

void Http2RequestClient::stop()
{
    server->stop();
}

std::unique_ptr<Http2RequestClient> Http2RequestClient::create(RequestClientImplementation *impl)
{
    return std::unique_ptr<Http2RequestClient>(new Http2RequestClient(impl));
}
#endif