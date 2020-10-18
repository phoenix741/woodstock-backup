#pragma once

#include <QString>
#include <QList>
#include <QByteArray>
#include <QSharedDataPointer>

#include "woodstock.pb.h"

namespace Common
{
    class FileManifestStatData;
    class FileManifestData;
    class JournalEntryData;

    class FileManifestStat
    {
    public:
        FileManifestStat();
        FileManifestStat(const FileManifestStat &other);
        ~FileManifestStat();

        void setSize(qint64 value);
        qint64 size() const;

        void setLastRead(qint64 value);
        qint64 lastRead() const;

        void setLastModified(qint64 value);
        qint64 lastModified() const;

        void setCreated(qint64 value);
        qint64 created() const;

        void setOwnerId(qint64 value);
        qint64 ownerId() const;

        void setGroupId(qint64 value);
        qint64 groupId() const;

        void setMode(qint64 value);
        qint64 mode() const;

    private:
        QSharedDataPointer<FileManifestStatData> d;
    };

    class FileManifest
    {
    public:
        FileManifest();
        FileManifest(const QString &path, qint64 size, qint64 lastModified);
        FileManifest(const FileManifest &other);
        ~FileManifest();

        bool operator==(const FileManifest &other) const;
        bool operator!=(const FileManifest &other) const;

        bool isValid() const;

        void setPath(const QString &value);
        const QString &path() const;

        void setStats(const FileManifestStat &value);
        FileManifestStat &stats();
        const FileManifestStat &stats() const;

        void setChunks(const QList<QByteArray> &value);
        QList<QByteArray> &chunks();
        const QList<QByteArray> &chunks() const;

        void setSha256(const QByteArray &value);
        QByteArray &sha256();
        const QByteArray &sha256() const;

        static FileManifest fromProtobuf(const woodstock::FileManifest &protoFileManifest);
        void toProtobuf(woodstock::FileManifest *protoFileManifest) const;

    private:
        QSharedDataPointer<FileManifestData> d;
    };

    enum JournalEntryType
    {
        ADD,
        MODIFY,
        REMOVE,
        CLOSE
    };

    class JournalEntry
    {
    public:
        JournalEntry();
        JournalEntry(const JournalEntry &other);
        ~JournalEntry();

        void setPath(const QString &path);
        const QString &path() const;

        void setType(const JournalEntryType &value);
        const JournalEntryType &type() const;

        void setManifest(const FileManifest &value);
        FileManifest &manifest();
        const FileManifest &manifest() const;

        static JournalEntry fromProtobuf(const woodstock::FileManifestJournalEntry &proto);
        void toProtobuf(woodstock::FileManifestJournalEntry *protoFileManifest) const;

    private:
        QSharedDataPointer<JournalEntryData> d;
    };

} // namespace Common

Q_DECLARE_TYPEINFO(Common::FileManifestStat, Q_MOVABLE_TYPE);
Q_DECLARE_TYPEINFO(Common::FileManifest, Q_MOVABLE_TYPE);
Q_DECLARE_TYPEINFO(Common::JournalEntry, Q_MOVABLE_TYPE);
