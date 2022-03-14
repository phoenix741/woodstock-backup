#include "request-server.h"

#include "woodstock.grpc.pb.h"

class GrpcRequestServer : public RequestServer
{
public:
    GrpcRequestServer(const QString &host);

    static std::unique_ptr<GrpcRequestServer> create(const QString &host);

    virtual PrepareResult prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid) override;
    virtual void refreshCache(ManifestWrapper *wrapper) override;
    virtual std::unique_ptr<Backup> updateFileManifest() override;
    virtual void launchBackup(qint32 backupNumber, std::function<void(const Common::JournalEntry &)> process) override;
    virtual qint64 getChunk(const QString filename, qint64 pos, qint64 size, PoolChunkWrapper *device) override;

private:
    std::unique_ptr<woodstock::WoodstockClientService::Stub> m_stub;
};
