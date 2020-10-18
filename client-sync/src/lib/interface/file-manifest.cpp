#include <QDebug>
#include "file-manifest.h"
#include "file_manifest_p.h"

namespace Common
{
    /* FileManifestStat */

    FileManifestStat::FileManifestStat() : d(new FileManifestStatData())
    {
    }

    FileManifestStat::FileManifestStat(const FileManifestStat &other) : d(other.d)
    {
    }

    FileManifestStat::~FileManifestStat()
    {
    }

    void FileManifestStat::setSize(qint64 value)
    {
        d->size = value;
    }

    qint64 FileManifestStat::size() const
    {
        return d->size;
    }

    void FileManifestStat::setLastRead(qint64 value)
    {
        d->lastRead = value;
    }

    qint64 FileManifestStat::lastRead() const
    {
        return d->lastRead;
    }

    void FileManifestStat::setLastModified(qint64 value)
    {
        d->lastModified = value;
    }

    qint64 FileManifestStat::lastModified() const
    {
        return d->lastModified;
    }

    void FileManifestStat::setCreated(qint64 value)
    {
        d->created = value;
    }

    qint64 FileManifestStat::created() const
    {
        return d->created;
    }

    void FileManifestStat::setOwnerId(qint64 value)
    {
        d->ownerId = value;
    }

    qint64 FileManifestStat::ownerId() const
    {
        return d->ownerId;
    }

    void FileManifestStat::setGroupId(qint64 value)
    {
        d->groupId = value;
    }

    qint64 FileManifestStat::groupId() const
    {
        return d->groupId;
    }

    void FileManifestStat::setMode(qint64 value)
    {
        d->mode = value;
    }

    qint64 FileManifestStat::mode() const
    {
        return d->mode;
    }

    /* FileManifest */

    FileManifest::FileManifest() : d(new FileManifestData())
    {
    }

    FileManifest::FileManifest(const QString &path, qint64 size, qint64 lastModified) : d(new FileManifestData())
    {
        d->path = path;
        d->stats.setSize(size);
        d->stats.setLastModified(lastModified);
    }

    FileManifest::FileManifest(const FileManifest &other) : d(other.d)
    {
    }

    FileManifest::~FileManifest()
    {
    }

    bool FileManifest::operator==(const FileManifest &other) const
    {
        return stats().lastModified() == other.stats().lastModified() && stats().size() == other.stats().size();
    }

    bool FileManifest::operator!=(const FileManifest &other) const
    {
        return !operator==(other);
    }

    bool FileManifest::isValid() const
    {
        return !d->path.isEmpty();
    }

    void FileManifest::setPath(const QString &value)
    {
        d->path = value;
    }

    const QString &FileManifest::path() const
    {
        return d->path;
    }

    void FileManifest::setStats(const FileManifestStat &value)
    {
        d->stats = value;
    }

    FileManifestStat &FileManifest::stats()
    {
        return d->stats;
    }

    const FileManifestStat &FileManifest::stats() const
    {
        return d->stats;
    }

    void FileManifest::setChunks(const QList<QByteArray> &value)
    {
        d->chunks = value;
    }

    QList<QByteArray> &FileManifest::chunks()
    {
        return d->chunks;
    }

    const QList<QByteArray> &FileManifest::chunks() const
    {
        return d->chunks;
    }

    void FileManifest::setSha256(const QByteArray &value)
    {
        d->sha256 = value;
    }

    QByteArray &FileManifest::sha256()
    {
        return d->sha256;
    }

    const QByteArray &FileManifest::sha256() const
    {
        return d->sha256;
    }

    FileManifest FileManifest::fromProtobuf(const woodstock::FileManifest &protoFileManifest)
    {
        FileManifest manifest;
        manifest.setPath(QByteArray::fromStdString(protoFileManifest.path()));
        manifest.setSha256(QByteArray::fromStdString(protoFileManifest.sha256()));
        for (auto i = 0; i < protoFileManifest.chunks_size(); i++)
        {
            manifest.chunks().push_back(QByteArray::fromStdString(protoFileManifest.chunks(i)));
        }

        /* Linux */

        manifest.stats().setOwnerId(protoFileManifest.stats().ownerid());
        manifest.stats().setGroupId(protoFileManifest.stats().groupid());
        manifest.stats().setSize(protoFileManifest.stats().size());
        manifest.stats().setLastRead(protoFileManifest.stats().lastread());
        manifest.stats().setLastModified(protoFileManifest.stats().lastmodified());
        manifest.stats().setCreated(protoFileManifest.stats().created());
        manifest.stats().setMode(protoFileManifest.stats().mode());

        return manifest;
    }

    void FileManifest::toProtobuf(woodstock::FileManifest *protoFileManifest) const
    {
        protoFileManifest->set_path(path().toStdString());
        protoFileManifest->set_sha256(sha256().data(), sha256().size());
        for (auto chunk : chunks())
        {
            protoFileManifest->add_chunks(chunk.data(), chunk.size());
        }

        /* Linux */

        protoFileManifest->mutable_stats()->set_ownerid(stats().ownerId());
        protoFileManifest->mutable_stats()->set_groupid(stats().groupId());
        protoFileManifest->mutable_stats()->set_size(stats().size());
        protoFileManifest->mutable_stats()->set_lastread(stats().lastRead());
        protoFileManifest->mutable_stats()->set_lastmodified(stats().lastModified());
        protoFileManifest->mutable_stats()->set_created(stats().created());
        protoFileManifest->mutable_stats()->set_mode(stats().mode());
    }

    /* JournalEntry */

    JournalEntry::JournalEntry() : d(new JournalEntryData())
    {
    }

    JournalEntry::JournalEntry(const JournalEntry &other) : d(other.d)
    {
    }

    JournalEntry::~JournalEntry()
    {
    }

    void JournalEntry::setPath(const QString &path)
    {
        d->path = path;
    }

    const QString &JournalEntry::path() const
    {
        return d->path;
    }

    void JournalEntry::setType(const JournalEntryType &value)
    {
        d->type = value;
    }

    const JournalEntryType &JournalEntry::type() const
    {
        return d->type;
    }

    void JournalEntry::setManifest(const FileManifest &value)
    {
        d->manifest = value;
    }

    FileManifest &JournalEntry::manifest()
    {
        return d->manifest;
    }

    const FileManifest &JournalEntry::manifest() const
    {
        return d->manifest;
    }

    JournalEntry JournalEntry::fromProtobuf(const woodstock::FileManifestJournalEntry &proto)
    {
        JournalEntry entry;
        switch (proto.type())
        {
        case woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_ADD:
            entry.setType(ADD);
            entry.setManifest(FileManifest::fromProtobuf(proto.manifest()));
            break;
        case woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_MODIFY:
            entry.setType(MODIFY);
            entry.setManifest(FileManifest::fromProtobuf(proto.manifest()));
            break;
        case woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_REMOVE:
            entry.setType(REMOVE);
            entry.setPath(QString::fromStdString(proto.path()));

            break;
        case woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_CLOSE:
            entry.setType(CLOSE);
            entry.setPath(QString::fromStdString(proto.path()));

            break;
        default:
            break;
        }

        return entry;
    }

    void JournalEntry::toProtobuf(woodstock::FileManifestJournalEntry *protoFileManifest) const
    {
        switch (type())
        {
        case ADD:
            protoFileManifest->set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_ADD);
            manifest().toProtobuf(protoFileManifest->mutable_manifest());
            break;
        case MODIFY:
            protoFileManifest->set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_MODIFY);
            manifest().toProtobuf(protoFileManifest->mutable_manifest());
            break;
        case REMOVE:
            protoFileManifest->set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_REMOVE);
            protoFileManifest->set_path(path().toStdString());
            break;
        case CLOSE:
            protoFileManifest->set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_CLOSE);
            protoFileManifest->set_path(path().toStdString());
            break;
        }
    }
} // namespace Common
