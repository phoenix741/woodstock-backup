#include "request-server.h"

#include <nghttp2/asio_http2_client.h>

using boost::asio::ip::tcp;

using namespace nghttp2::asio_http2;
using namespace nghttp2::asio_http2::client;

class Http2RequestServer : public RequestServer
{
public:
    Http2RequestServer(const QString &host);

    static std::unique_ptr<Http2RequestServer> create(const QString &host);

    virtual PrepareResult prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid) override;
    virtual void refreshCache(ManifestWrapper *wrapper) override;
    virtual std::unique_ptr<Backup> updateFileManifest() override;
    virtual void launchBackup(qint32 backupNumber, std::function<void(const Common::JournalEntry &)> process) override;
    virtual qint64 getChunk(const QString filename, qint64 pos, qint64 size, PoolChunkWrapper *device) override;

private:
    boost::system::error_code m_ec;
    boost::asio::ssl::context m_tls;
    std::unique_ptr<boost::asio::io_service> m_service;
    std::unique_ptr<session> m_session;

    QString m_host;
};
