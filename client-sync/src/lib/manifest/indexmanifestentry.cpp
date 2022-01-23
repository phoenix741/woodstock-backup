#include "indexmanifestentry.h"
#include "indexmanifestentry_p.h"
#include "../interface/file-manifest.h"

IndexFileEntry::IndexFileEntry() : d(new IndexFileEntryData())
{
}

IndexFileEntry::IndexFileEntry(const QString &path) : d(new IndexFileEntryData())
{
    setPath(path);
}

IndexFileEntry::IndexFileEntry(const IndexFileEntry &other) : d(other.d)
{
}

IndexFileEntry::~IndexFileEntry()
{
}

IndexFileEntry &IndexFileEntry::operator=(const IndexFileEntry &rhs)
{
    if (this == &rhs)
        return *this; //Protect against self-assignment
    d = rhs.d;
    return *this;
}

bool IndexFileEntry::isValid() const
{
    return !d->path.isEmpty();
}

void IndexFileEntry::setPath(const QString &value)
{
    d->path = value;
}

const QString &IndexFileEntry::path() const
{
    return d->path;
}

void IndexFileEntry::setJournal(bool journal)
{
    d->journal = journal;
}

bool IndexFileEntry::journal() const
{
    return d->journal;
}

void IndexFileEntry::setIndex(qint64 value)
{
    d->index = value;
}

qint64 IndexFileEntry::index() const
{
    return d->index;
}

void IndexFileEntry::setRead(bool value)
{
    d->read = value;
}

bool IndexFileEntry::read() const
{
    return d->read;
}

void IndexFileEntry::setDeleted(bool value)
{
    d->deleted = value;
}

bool IndexFileEntry::deleted() const
{
    return d->deleted;
}

void IndexFileEntry::setFiles(QHash<QString, IndexFileEntry> value)
{
    d->files = value;
}

const QHash<QString, IndexFileEntry> &IndexFileEntry::files() const
{
    return d->files;
}

QHash<QString, IndexFileEntry> &IndexFileEntry::files()
{
    return d->files;
}

void IndexFileEntry::setLastModifiedDate(qint64 value)
{
    d->lastModifiedDate = value;
}

qint64 IndexFileEntry::lastModifiedDate() const
{
    return d->lastModifiedDate;
}

void IndexFileEntry::setSize(qint64 value)
{
    d->size = value;
}

qint64 IndexFileEntry::size() const
{
    return d->size;
}

Common::FileManifest IndexFileEntry::toManifest() const
{
    return Common::FileManifest(path(), size(), lastModifiedDate());
}