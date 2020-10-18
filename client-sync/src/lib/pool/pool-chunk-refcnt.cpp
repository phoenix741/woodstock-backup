#include "pool-chunk-refcnt.h"

#include <QDebug>
#include <QLockFile>
#include <QDataStream>
#include <QDirIterator>

PoolChunkRefCnt::PoolChunkRefCnt(const QString &poolPath) : m_poolPath(poolPath)
{
}

std::unique_ptr<PoolChunkRefCnt> PoolChunkRefCnt::get(const QString &poolPath)
{
    return std::unique_ptr<PoolChunkRefCnt>(new PoolChunkRefCnt(poolPath));
}

qint64 PoolChunkRefCnt::incr(const QByteArray &sha256)
{
    QLockFile lockfile(getLockFile(sha256.toHex()));
    if (lockfile.lock())
    {
        auto result = readFile(sha256);
        auto cnt = result.value(sha256, 0) + 1;
        result.insert(sha256, cnt);
        writeFile(sha256, result);
        return cnt;
    }
    return -1;
}

qint64 PoolChunkRefCnt::decr(const QByteArray &sha256)
{
    QLockFile lockfile(getLockFile(sha256.toHex()));
    if (lockfile.lock())
    {
        auto result = readFile(sha256);
        auto cnt = result.value(sha256, 0) - 1;
        if (cnt < 0)
        {
            qWarning() << "SHA256 is already pending for deletion";
        }
        result.insert(sha256, cnt);
        writeFile(sha256, result);
        return cnt - 1;
    }
    return -1;
}

QHash<QByteArray, qint64> PoolChunkRefCnt::readFile(const QByteArray &sha256) const
{
    QHash<QByteArray, qint64> result;
    QFile refcntFile(getRefCnt(sha256.toHex()));
    if (refcntFile.open(QFile::ReadOnly))
    {
        QDataStream in(&refcntFile);
        in >> result;
    }
    return result;
}

void PoolChunkRefCnt::writeFile(const QByteArray &sha256, QHash<QByteArray, qint64> map) const
{
    QHash<QByteArray, qint64> result;
    QFile refcntFile(getRefCnt(sha256.toHex()));
    if (!refcntFile.open(QFile::WriteOnly))
    {
        throw std::exception(); // FIXME: Can't open the file
    }
    QDataStream out(&refcntFile);
    out << result;
}

QList<QByteArray> PoolChunkRefCnt::cleanUp()
{
    QDirIterator it(m_poolPath.absolutePath(), QStringList() << "REFCNT", QDir::NoFilter, QDirIterator::Subdirectories);
    QList<QByteArray> result;

    // Iterate through the directory using the QDirIterator
    while (it.hasNext())
    {
        QString filename = it.next();
        QLockFile lockfile(filename + ".lock");
        QHash<QByteArray, qint64> data;
        if (lockfile.lock())
        {
            QFile refcntFile(filename);
            if (!refcntFile.open(QFile::ReadOnly))
            {
                throw std::exception(); // FIXME: Can't open the file
            }
            QDataStream in(&refcntFile);
            in >> data;
        }

        QHash<QByteArray, qint64>::iterator i;
        for (auto i = data.begin(); i != data.end(); ++i)
        {
            if (i.value() <= 0)
            {
                result.append(i.key());
            }
        }
    }

    return result;
}

// FIXME: Chunk dir should be refactord with PoolChunkWrapper
QString PoolChunkRefCnt::getChunkDir(const QString &sha256) const
{
    QStringRef part1 = sha256.midRef(0, 2);
    QStringRef part2 = sha256.midRef(2, 2);
    QStringRef part3 = sha256.midRef(4, 2);

    return m_poolPath.absoluteFilePath(part1 + QDir::separator() + part2 + QDir::separator() + part3);
}

QString PoolChunkRefCnt::getLockFile(const QString &sha256) const
{
    return QDir(getChunkDir(sha256)).absoluteFilePath("REFCNT.lock");
}

QString PoolChunkRefCnt::getRefCnt(const QString &sha256) const
{
    return QDir(getChunkDir(sha256)).absoluteFilePath("REFCNT");
}
