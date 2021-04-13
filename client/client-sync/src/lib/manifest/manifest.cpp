#include "manifest.h"
#include "woodstock.pb.h"
#include "../utils/sha256.h"
#include "../protobuf/group_grpc.h"
#include "manifest_wrapper.h"
#include "indexmanifest.h"

#include <QDir>
#include <QDebug>
#include <fstream>
#include <sys/stat.h>

Manifest::Manifest(const QString &manifestName, const QString &path)
    : m_journalPath(path + "/" + manifestName + ".journal"),
      m_manifestPath(path + "/" + manifestName + ".manifest"),
      m_indexPath(path + "/" + manifestName + ".index"),
      m_newPath(path + "/" + manifestName + ".new"),
      m_lockPath(path + "/" + manifestName + ".lock")
{
    QDir(path).mkpath(".");
}

Manifest::~Manifest()
{
}

void Manifest::deleteManifest()
{
    if (QFile::exists(m_newPath))
    {
        QFile::remove(m_newPath);
    }
    if (QFile::exists(m_indexPath))
    {
        QFile::remove(m_indexPath);
    }
    if (QFile::exists(m_journalPath))
    {
        QFile::remove(m_journalPath);
    }
    if (QFile::exists(m_manifestPath))
    {
        QFile::remove(m_manifestPath);
    }
}

const QString &Manifest::lockPath() const
{
    return m_lockPath;
}

ManifestWrapper *Manifest::getManifestWrapper(const QString &path)
{
    if (!m_manifestFiles[path])
    {
        m_manifestFiles[path] = std::unique_ptr<ManifestWrapper>(new ManifestWrapper(path));
    }
    return m_manifestFiles[path].get();
}

ManifestWrapper *Manifest::getManifestWrapper()
{
    return getManifestWrapper(m_manifestPath);
}

void Manifest::closeManifest(const QString &path)
{
    if (m_manifestFiles[path])
    {
        m_manifestFiles[path]->close();
        m_manifestFiles.erase(path);
    }
}

std::unique_ptr<IndexManifest> Manifest::loadIndex()
{
    auto index = new IndexManifest();
    try
    {
        ManifestWrapper inWrapper(m_manifestPath);
        inWrapper.readAllMessages<woodstock::FileManifest>([&](const woodstock::FileManifest &fileManifest, const std::streampos &pos) {
            auto entry = index->add(QString::fromStdString(fileManifest.path()), pos, false);
            entry.setLastModifiedDate(fileManifest.stats().lastmodified());
            entry.setSize(fileManifest.stats().size());
        });

        ManifestWrapper outWrapper(m_journalPath);
        outWrapper.readAllMessages<woodstock::FileManifestJournalEntry>([&](const woodstock::FileManifestJournalEntry &journalEntry, const std::streampos &pos) {
            if (journalEntry.type() != woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_REMOVE)
            {
                auto entry = index->add(QString::fromStdString(journalEntry.manifest().path()), pos, true);
                entry.setLastModifiedDate(journalEntry.manifest().stats().lastmodified());
                entry.setSize(journalEntry.manifest().stats().size());
            }
            else
            {
                index->remove(QString::fromStdString(journalEntry.path()));
            }
        });
    }
    catch (std::exception &exception)
    {
        throw exception;
    }

    return std::unique_ptr<IndexManifest>(index);
}

void Manifest::addManifest(const Common::FileManifest &manifest, bool add)
{
    woodstock::FileManifestJournalEntry entry;
    manifest.toProtobuf(entry.mutable_manifest());
    entry.set_type(
        add
            ? woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_ADD
            : woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_MODIFY);

    if (!getManifestWrapper(m_journalPath)->writeMessage<woodstock::FileManifestJournalEntry>(entry))
    {
        throw std::exception(); // TODO
    }
}

void Manifest::removePath(const QString &path)
{
    woodstock::FileManifestJournalEntry entry;
    entry.set_path(path.toStdString());
    entry.set_type(woodstock::FileManifestJournalEntry_EntryType::FileManifestJournalEntry_EntryType_REMOVE);

    if (!getManifestWrapper(m_journalPath)->writeMessage<woodstock::FileManifestJournalEntry>(entry))
    {
        throw std::exception(); // TODO
    }
}

void Manifest::compact(std::function<void(const Common::FileManifest &)> incrementFileCount)
{
    closeManifest(m_journalPath);

    auto indexManifest = loadIndex();
    auto newWrapper = std::unique_ptr<ManifestWrapper>(new ManifestWrapper(m_newPath));
    ManifestWrapper *newWrapperPtr = newWrapper.get();

    indexManifest->walk([this, newWrapperPtr, incrementFileCount](IndexFileEntry entry) {
        if (!entry.deleted())
        {
            auto fileManifest = getProtobufManifest(entry);
            if (fileManifest)
            {
                newWrapperPtr->writeMessage<woodstock::FileManifest>(*fileManifest.get());
                if (incrementFileCount)
                {
                    incrementFileCount(Common::FileManifest::fromProtobuf(*(fileManifest.get())));
                }
            }
            else if (entry.files().size() == 0)
            {
                qWarning() << "File manifest not present " << entry.path() << entry.journal() << entry.index() << entry.deleted() << entry.read();
            }
        }
    });
    newWrapper.reset();

    closeManifest(m_journalPath);
    closeManifest(m_manifestPath);

    if (QFile::exists(m_journalPath))
    {
        QFile::remove(m_journalPath);
    }
    if (QFile::exists(m_manifestPath))
    {
        QFile::remove(m_manifestPath);
    }
    QFile::rename(m_newPath, m_manifestPath);
}

Common::FileManifest Manifest::getManifest(IndexManifest *index, const QString &path)
{
    auto entry = index->getEntry(path);
    return getManifest(entry);
}

Common::FileManifest Manifest::getManifest(IndexFileEntry indexEntry)
{
    auto manifest = getProtobufManifest(indexEntry);
    if (manifest)
    {
        return Common::FileManifest::fromProtobuf(*(manifest.get()));
    }
    else
    {
        return Common::FileManifest();
    }
}

std::unique_ptr<woodstock::FileManifest> Manifest::getProtobufManifest(IndexFileEntry entry)
{
    if (entry.isValid() && entry.index() >= 0)
    {
        if (entry.journal())
        {
            woodstock::FileManifestJournalEntry journalEntry;
            auto wrapper = getManifestWrapper(m_journalPath);
            wrapper->readMessage<woodstock::FileManifestJournalEntry>(&journalEntry, entry.index());
            return std::unique_ptr<woodstock::FileManifest>(new woodstock::FileManifest(journalEntry.manifest()));
        }
        else
        {
            std::unique_ptr<woodstock::FileManifest> fileManifest(new woodstock::FileManifest());
            auto wrapper = getManifestWrapper(m_manifestPath);
            wrapper->readMessage<woodstock::FileManifest>(fileManifest.get(), entry.index());
            return fileManifest;
        }
    }
    return std::unique_ptr<woodstock::FileManifest>();
}

bool Manifest::compareManifest(const Common::FileManifest &fm1, const Common::FileManifest &fm2)
{
    return fm1.stats().lastModified() == fm2.stats().lastModified() && fm1.stats().size() == fm2.stats().size();
}

void Manifest::loadManifestChunk(Common::FileManifest &fm)
{
    if ((fm.stats().mode() & S_IFMT) == S_IFREG)
    {
        auto hash = sha256(fm.path());
        fm.setSha256(hash.shasum);
        fm.setChunks(hash.chunks);
    }
}
