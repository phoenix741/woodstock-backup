#pragma once

#include <QString>
#include <QHash>
#include <QExplicitlySharedDataPointer>

namespace Common
{
    class FileManifest;
}

class IndexFileEntryData;

class IndexFileEntry
{
public:
    IndexFileEntry();
    IndexFileEntry(const QString &path);
    IndexFileEntry(const IndexFileEntry &other);
    ~IndexFileEntry();

    IndexFileEntry &operator=(const IndexFileEntry &rhs);

    bool isValid() const;

    void setPath(const QString &value);
    const QString &path() const;

    void setJournal(bool journal);
    bool journal() const;

    void setIndex(qint64 value);
    qint64 index() const;

    void setRead(bool value);
    bool read() const;

    void setDeleted(bool value);
    bool deleted() const;

    void setFiles(QHash<QString, IndexFileEntry> value);
    const QHash<QString, IndexFileEntry> &files() const;
    QHash<QString, IndexFileEntry> &files();

    void setLastModifiedDate(qint64 value);
    qint64 lastModifiedDate() const;

    void setSize(qint64 value);
    qint64 size() const;

    Common::FileManifest toManifest() const;

private:
    QExplicitlySharedDataPointer<IndexFileEntryData> d;
};
