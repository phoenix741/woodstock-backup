#pragma once

#include <functional>
#include <memory>
#include <QString>
#include <QIODevice>

#include <interface/configuration.h>
#include <interface/file-manifest.h>

class RequestClientCache
{
public:
    virtual ~RequestClientCache();
    virtual void addFileManifest(const Common::FileManifest &manifest) = 0;
    virtual void close() = 0;
};

struct PrepareResult
{
    bool needRefreshCache;
};

class RequestClientImplementation
{
public:
    virtual ~RequestClientImplementation();

    virtual PrepareResult prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid) = 0;

    virtual std::unique_ptr<RequestClientCache> refreshCache() = 0;

    virtual void launchBackup(qint32 backupNumber, std::function<bool(const Common::JournalEntry &)> process) = 0;

    virtual std::unique_ptr<QIODevice> getChunk(const QString filename, qint64 pos) = 0;
};

class RequestClient
{
public:
    RequestClient(RequestClientImplementation *impl);
    virtual ~RequestClient();

    virtual void listen() = 0;
    virtual void stop() = 0;

protected:
    RequestClientImplementation *m_impl;
};
