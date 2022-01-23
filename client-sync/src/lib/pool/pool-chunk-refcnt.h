#pragma once

#include <memory>
#include <QString>
#include <QDir>
#include <QHash>

class PoolChunkRefCnt
{
public:
    static std::unique_ptr<PoolChunkRefCnt> get(const QString &poolPath);

    qint64 incr(const QByteArray &sha256);
    qint64 decr(const QByteArray &sha256);

    QList<QByteArray> cleanUp();

private:
    PoolChunkRefCnt(const QString &poolPath);

    QHash<QByteArray, qint64> readFile(const QByteArray &sha256) const;
    void writeFile(const QByteArray &sha256, QHash<QByteArray, qint64> map) const;

    QString getChunkDir(const QString &sha256) const;
    QString getLockFile(const QString &sha256) const;
    QString getRefCnt(const QString &sha256) const;

    QDir m_poolPath;
};
