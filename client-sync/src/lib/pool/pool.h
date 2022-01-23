#pragma once

#include <QString>
#include <memory>

class PoolChunkWrapper;

/**
 * This class should be thread safe because of multiple backup can be made in the
 * same time.
 * 
 * Two program can create the same chunk at the same time, or remove the same chunk
 * at the same time.
 * 
 * Structure of the pool
 * 
 * pool
 *   |___ aa
 *         |____aa
 *               |_____aa 
 *                      |______ UNUSED
 *                                  sha256 date
 *                      |______ REFCNT
 *                                  sha256 cnt
 *                                  sha256 cnt
 *                                  sha256 cnt
 *                      |______ LOCK
 *                                  host backupNumber
 *                      |______ aaaaaacdefghih-sha256.gz
 */
class BackupPool
{
public:
    BackupPool(const QString &poolPath);
    virtual ~BackupPool();

    std::unique_ptr<PoolChunkWrapper> getChunk(const QByteArray &sha256);

    bool isChunkExists(const QByteArray &sha256);

private:
    QString m_poolPath;
};
