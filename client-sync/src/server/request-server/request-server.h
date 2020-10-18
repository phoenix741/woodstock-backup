#pragma once

#include <memory>
#include <QString>
#include <interface/configuration.h>
#include <interface/file-manifest.h>
#include <pool/pool-chunk-wrapper.h>

class ManifestWrapper;

struct PrepareResult
{
    bool needRefreshCache;
};

class Backup
{
public:
    virtual void updateFileManifest(const Common::JournalEntry &entry) = 0;
};

class RequestServer
{
public:
    virtual ~RequestServer();

    virtual PrepareResult prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid) = 0;
    virtual void refreshCache(ManifestWrapper *wrapper) = 0;
    virtual std::unique_ptr<Backup> updateFileManifest() = 0;
    virtual void launchBackup(qint32 backupNumber, std::function<void(const Common::JournalEntry &)> process) = 0;
    virtual qint64 getChunk(const QString filename, qint64 pos, qint64 size, PoolChunkWrapper *device) = 0;
};
