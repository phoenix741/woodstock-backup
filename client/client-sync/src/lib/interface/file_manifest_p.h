#pragma once

#include <QString>
#include <QList>
#include <QByteArray>
#include <QSharedData>

#include "file-manifest.h"

namespace Common
{

    class FileManifestStatData : public QSharedData
    {
    public:
        FileManifestStatData()
            : size(-1),
              lastRead(0),
              lastModified(0),
              created(0),
              ownerId(-1),
              groupId(-1),
              mode(0) {}
        FileManifestStatData(const FileManifestStatData &other)
            : QSharedData(other),
              size(other.size),
              lastRead(other.lastRead),
              lastModified(other.lastModified),
              created(other.created),
              ownerId(other.ownerId),
              groupId(other.groupId),
              mode(other.mode) {}
        ~FileManifestStatData() {}

        qint64 size;
        qint64 lastRead;
        qint64 lastModified;
        qint64 created;

        /* Linux */
        qint64 ownerId;
        qint64 groupId;
        qint64 mode;
    };

    class FileManifestData : public QSharedData
    {
    public:
        FileManifestData() {}
        FileManifestData(const FileManifestData &other)
            : QSharedData(other),
              path(other.path),
              stats(other.stats),
              chunks(other.chunks),
              sha256(other.sha256) {}
        ~FileManifestData() {}

        QString path;
        FileManifestStat stats;
        QList<QByteArray> chunks;
        QByteArray sha256;
    };

    class JournalEntryData : public QSharedData
    {
    public:
        JournalEntryData() {}
        JournalEntryData(const JournalEntryData &other)
            : QSharedData(other),
              type(other.type),
              manifest(other.manifest),
              path(other.path) {}
        ~JournalEntryData() {}

        JournalEntryType type;
        FileManifest manifest;
        QString path;
    };
}; // namespace Common