#include "pool.h"

#include "pool-chunk-wrapper.h"

BackupPool::BackupPool(const QString &poolPath) : m_poolPath(poolPath)
{
}

BackupPool::~BackupPool()
{
}

std::unique_ptr<PoolChunkWrapper> BackupPool::getChunk(const QByteArray &sha256)
{
    if (!PoolChunkWrapper::exists(m_poolPath, sha256))
    {
        return PoolChunkWrapper::create(m_poolPath, sha256);
    }
    return PoolChunkWrapper::get(m_poolPath, sha256);
}

bool BackupPool::isChunkExists(const QByteArray &sha256)
{
    return PoolChunkWrapper::exists(m_poolPath, sha256);
}