#pragma once

#include <functional>
#include <unordered_map>
#include <QString>
#include "../interface/file-manifest.h"
#include "../utils/qstringhash.h"

class ManifestWrapper;
class IndexManifest;
class IndexFileEntry;

namespace woodstock
{
    class FileManifest;
}

/** 
 * Le but de cette classe est de gére le fichier manifest.
 * 
 * Ce manifest est constitué de trois fichiers :
 * - backup.manifest: list des informations sur chaques fichiers
 * - backup.index: structure des fichiers et des repertoires avec le positionnement dans le fichier manifest.
 *   (Optionel) : Combien de temps il faut pour lire le fichier manifest et reconstuire l'arbre en mémoire ?
 *   Lors du parcours des fichiers dans l'arborescence, est-ce assez rapide ?
 * - backup.journal: Fichier listant les ajouts, modification et suppression à faire dans le journal.
 * 
 */
class Manifest
{
public:
    Manifest(const QString &manifestName, const QString &path);
    ~Manifest();

    /**
     * @brief Delete the manifest
     */
    void deleteManifest();

    /**
     * Charge l'index en mémoire
     */
    std::unique_ptr<IndexManifest> loadIndex();

    /**
     * Ajoute une entrée dans le journal
     * 
     * @param add true if the file is added, false if the file is modified
     */
    void addManifest(const Common::FileManifest &manifest, bool add);

    /**
     * Ajoute une entrée dans le journal
     */
    void removePath(const QString &path);

    /**
     * Lance le processus bloquant créant un nouveau fichier manifest.
     * 
     * - Charge l'index en mémoire
     * - Applique le journal à l'index (avec pointeur sur le journal)
     * - Sauvegarde le nouveau manifest dans un nouveau fichier à partir de l'index.
     * - Ecrase le fichier original
     * - Sauvegarde l'index ?
     */
    void compact(std::function<void(const Common::FileManifest &)> incrementFileCount = {});

    /**
     * Récupère dans le manifest le fichier demandé
     */
    Common::FileManifest getManifest(IndexManifest *index, const QString &path);

    /**
     * Récupère dans le manifest le fichier demandé
     */
    Common::FileManifest getManifest(IndexFileEntry indexEntry);

    /**
     * Compare deux manifests enssemble.
     * 
     * La comparaison va se faire sur la date de création, de modification, la taille du fichier.
     */
    static bool compareManifest(const Common::FileManifest &fm1, const Common::FileManifest &fm2);

    /**
     * Lit un fichier pour remplir le manifest avec les sha256.
     */
    static void loadManifestChunk(Common::FileManifest &fm);

    const QString &lockPath() const;

    ManifestWrapper *getManifestWrapper();

private:
    std::unique_ptr<woodstock::FileManifest> getProtobufManifest(IndexFileEntry indexEntry);
    ManifestWrapper *getManifestWrapper(const QString &path);
    void closeManifest(const QString &path);

    std::unordered_map<QString, std::unique_ptr<ManifestWrapper>> m_manifestFiles;

    QString m_journalPath, m_manifestPath, m_indexPath, m_newPath, m_lockPath;
};
