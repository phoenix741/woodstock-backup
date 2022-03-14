#include "indexmanifest.h"

#include <QPair>
#include <QDebug>
#include <functional>
#include <algorithm>
#include "../interface/file-manifest.h"

QPair<IndexFileEntry, bool> searchEntry(IndexFileEntry index, const QStringList &filePath, bool insert = false, int filePathIndex = 0)
{
    const auto pathPart = filePath[filePathIndex];

    IndexFileEntry entry = index.files().value(pathPart);

    if (!entry.isValid())
    {
        if (insert)
        {
            entry.setPath(pathPart);
            index.files().insert(pathPart, entry);
        }
        else
        {
            return QPair<IndexFileEntry, bool>(IndexFileEntry(), false);
        }
    }

    if ((filePathIndex + 1) >= filePath.size())
    {
        return QPair<IndexFileEntry, bool>(entry, true);
    }

    return searchEntry(entry, filePath, insert, filePathIndex + 1);
}

void listFile(const QString &prefix, QStringList &files, IndexFileEntry index, std::function<bool(IndexFileEntry)> filter)
{
    for (auto entry = index.files().begin(); entry != index.files().end(); ++entry)
    {
        auto path = prefix;

        path.reserve(prefix.length() + entry.value().path().length() + 1);
        path.append("/");
        path.append(entry.value().path());
        if (filter(entry.value()))
        {
            files.push_back(path);
        }
        listFile(path, files, entry.value(), filter);
    }
}

void walkFile(IndexFileEntry index, std::function<void(IndexFileEntry)> run)
{
    for (auto entry = index.files().begin(); entry != index.files().end(); ++entry)
    {
        run(entry.value());
        walkFile(entry.value(), run);
    }
}

/* IndexManifest */

IndexManifest::IndexManifest() : indexSize(0)
{
}

IndexFileEntry IndexManifest::add(const QString &filePath, qint64 index, bool isJournal)
{
    const auto path = filePath.split('/', QString::SkipEmptyParts);
    auto entry = searchEntry(rootIndex, path, true);

    if (!entry.first.isValid())
    {
        throw std::exception(); // FIXME
    }

    entry.first.setRead(false);
    entry.first.setDeleted(false);
    entry.first.setJournal(isJournal);
    entry.first.setIndex(index);
    indexSize++;

    return entry.first;
}

void IndexManifest::remove(const QString &filePath)
{
    const auto path = filePath.split('/', QString::SkipEmptyParts);
    auto entry = searchEntry(rootIndex, path, true);

    if (!entry.first.isValid())
    {
        throw std::exception(); // FIXME
    }

    entry.first.setRead(false);
    entry.first.setDeleted(true);
}

void IndexManifest::mark(IndexFileEntry entry)
{
    if (entry.isValid())
    {
        entry.setRead(true);
    }
}

QStringList IndexManifest::unmarkedFile()
{
    QStringList files;
    listFile("", files, rootIndex, [](IndexFileEntry entry) { return !entry.read(); });
    return files;
}

void IndexManifest::walk(std::function<void(IndexFileEntry)> run)
{
    walkFile(rootIndex, run);
}

IndexFileEntry IndexManifest::getEntry(const QString &filePath)
{
    if (filePath.isEmpty())
    {
        return rootIndex;
    }

    const auto path = filePath.split('/', QString::SkipEmptyParts);

    return searchEntry(rootIndex, path).first;
}
