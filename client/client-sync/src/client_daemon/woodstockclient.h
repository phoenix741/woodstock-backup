#pragma once

#include <memory>
#include "woodstock.grpc.pb.h"
#include "clientconfig.h"
#include <interface/file-manifest.h>
#include <interface/configuration.h>
#include <QElapsedTimer>

#include "request-client/request-client.h"

class Manifest;
class IndexManifest;
class RequestClientBackup;

class WoodstockClient : protected RequestClientImplementation
{
public:
    explicit WoodstockClient();
    virtual ~WoodstockClient();

    virtual void listen();
    virtual void stop();

protected:
    virtual PrepareResult prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid) override;
    virtual std::unique_ptr<RequestClientCache> refreshCache() override;
    virtual void launchBackup(qint32 backupNumber, std::function<bool(const Common::JournalEntry &)> process) override;
    virtual std::unique_ptr<QIODevice> getChunk(const QString filename, qint64 pos) override;

private:
    void printStats();

    void processTask(IndexManifest *index, Manifest *manifest, std::function<bool(const Common::JournalEntry &)> process, const Common::Task &task);
    void processCommand(const Common::Task &task);

    std::unique_ptr<RequestClient> m_server;
    std::unique_ptr<ClientConfig> m_config;

    qint64 m_lastElapsed;
    QElapsedTimer m_timer;
    qint64 nbFileRead, nbFileError, totalSize, transferredSize;

    Common::Configuration m_currentConfiguration;
    qint32 m_currentBackupId;
};
