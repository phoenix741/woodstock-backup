#pragma once

#include <QString>
#include <QStringList>
#include <manifest/indexmanifestentry.h>
#include <functional>

class Manifest;

class IndexManifest
{
public:
    IndexManifest();

    /**
     * Marque un fichier comme vu.
     * 
     * Retourn true si le fichier est déjà présent et false sinon.
     */
    void mark(IndexFileEntry entry);

    /**
     * Calculate and return the list of file not marked, and that can be 
     * mark for deletion.
     */
    QStringList unmarkedFile();

    /**
     * Read all the file in the walk;
     * @param filter 
     */
    void walk(std::function<void(IndexFileEntry)> run);

    /**
     * Get the index entry for the given file path;
     */
    IndexFileEntry getEntry(const QString &filePath);

    qint64 indexSize;

private:
    IndexFileEntry add(const QString &path, qint64 index, bool isJournal);
    void remove(const QString &path);

    IndexFileEntry rootIndex;
    friend class Manifest;
};
