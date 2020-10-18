#ifdef WITH_PROTOCOL_HTTP2
#include "http2-request-server.h"
#include "http2-request-server-exception.h"
#include "protobuf/group_grpc.h"

#include <QDebug>
#include <QFile>
#include <QBuffer>
#include <manifest/manifest_wrapper.h>

#include <utils/sha256.h>
#include <utils/readfile.h>

#include <boost/date_time/posix_time/posix_time.hpp>

using namespace boost::posix_time;

/* Http2RequestServer */

Http2RequestServer::Http2RequestServer(const QString &host) : m_tls(boost::asio::ssl::context::sslv23), m_service(new boost::asio::io_service()), m_host(host)
{
    m_tls.set_default_verify_paths();
    // disabled to make development easier...
    // tls_ctx.set_verify_mode(boost::asio::ssl::verify_peer);
    configure_tls_context(m_ec, m_tls);

    m_session = std::unique_ptr<session>(new session(*(m_service.get()), m_tls, host.toStdString(), "3000"));
    m_session->read_timeout(time_duration(3, 0, 0, 0));
    m_session->on_error([](const boost::system::error_code &ec) {
        std::cerr << "error: " << ec.message() << std::endl;
    });
}

std::unique_ptr<Http2RequestServer> Http2RequestServer::create(const QString &host)
{
    return std::unique_ptr<Http2RequestServer>(new Http2RequestServer(host));
}

PrepareResult Http2RequestServer::prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid)
{
    woodstock::PrepareBackupReply reply;
    std::string buffer;
    bool finished = false;
    m_session->on_connect([this, &buffer, &reply, &finished, &configuration, &lastBackupUuid, &newBackupUuid](tcp::resolver::iterator endpoint_it) {
        woodstock::PrepareBackupRequest request;
        configuration.toProtobuf(request.mutable_configuration());
        // qDebug() << QString::fromStdString(request.DebugString());

        std::string data = request.SerializeAsString();

        boost::system::error_code ec;
        auto req = m_session->submit(ec, "POST", QString("https://%1:3000/prepare-backup").arg(m_host).toStdString(), data);

        req->on_response([&buffer](const response &res) {
            res.on_data([&buffer](const uint8_t *p, std::size_t size) {
                buffer.append((const char *)p, size);
            });
        });
        req->on_close([&buffer, &reply, &finished](uint32_t error_code) {
            finished = true;

            if (!reply.ParseFromString(buffer))
            {
                throw Http2RequestServerException("Preparation of backup has been cancelled by client");
            }
        });
    });

    while (!finished)
    {
        m_service->run_one();
    }

    auto statusCode = reply.code();
    auto needRefreshCache = reply.needrefreshcache();

    if (statusCode == woodstock::StatusCode::Failed)
    {
        throw Http2RequestServerException("Preparation of backup has been failed");
    }

    return {needRefreshCache};
}

void Http2RequestServer::refreshCache(ManifestWrapper *wrapper)
{
}

std::unique_ptr<Backup> Http2RequestServer::updateFileManifest()
{
    return NULL;
}

void Http2RequestServer::launchBackup(qint32 backupNumber, std::function<void(const Common::JournalEntry &)> process)
{
    std::unique_ptr<QBuffer> inputBuffer = std::unique_ptr<QBuffer>(new QBuffer());
    if (!inputBuffer->open(QIODevice::ReadWrite))
    {
        throw Http2RequestServerException("Can't open memory buffer");
    }

    woodstock::FileManifestJournalEntry journalEntry;
    bool finished = false;
    woodstock::LaunchBackupRequest request;
    request.set_backupnumber(backupNumber);

    std::string data;
    request.SerializeToString(&data);

    boost::system::error_code ec;
    auto req = m_session->submit(ec, "POST", QString("https://%1:3000/launch-backup").arg(m_host).toStdString(), data);

    req->on_response([&inputBuffer](const response &res) {
        res.on_data([&inputBuffer](const uint8_t *p, std::size_t size) {
            inputBuffer->write((const char *)p, size);
        });
    });
    req->on_close([&finished](uint32_t error_code) {
        finished = true;
    });

    while (!finished)
    {
        m_service->run_one();
    }

    inputBuffer->seek(0);

    while (auto cont = readDelimitedFrom(inputBuffer.get(), &journalEntry))
    {
        auto entry = Common::JournalEntry::fromProtobuf(journalEntry);

        // qDebug() << "read " << entry.path() << entry.manifest().path();

        process(entry);
    }
}

qint64 Http2RequestServer::getChunk(const QString filename, qint64 pos, qint64 size, PoolChunkWrapper *device)
{
    // qDebug() << "get " << filename << pos << size;

    bool finished = false;
    woodstock::GetChunkRequest request;

    request.set_filename(filename.toStdString());
    request.set_position(pos);
    request.set_size(size);
    request.set_sha256(device->sha256().toStdString());

    std::string data;
    request.SerializeToString(&data);

    boost::system::error_code ec;
    auto req = m_session->submit(ec, "POST", QString("https://%1:3000/get-chunk").arg(m_host).toStdString(), data);

    qint64 totalRead = 0;

    req->on_response([device, &totalRead](const response &res) {
        res.on_data([device, &totalRead](const uint8_t *p, std::size_t size) {
            device->getDevice()->write((const char *)p, size);
            totalRead += size;
        });
    });
    req->on_close([&finished](uint32_t error_code) {
        finished = true;
    });

    while (!finished)
    {
        m_service->run_one();
    }

    return totalRead;
}

#endif