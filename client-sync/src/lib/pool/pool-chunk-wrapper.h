#pragma once

#include <memory>
#include <QDir>
#include <QString>
#include <QLockFile>

class QIODevice;

class PoolChunkWrapper
{
public:
    virtual ~PoolChunkWrapper();

    /**
     * Check if the chunk exists in the pool.
     * @returns true if the file exists
     */
    static bool exists(const QString &poolPath, const QByteArray &sha256);

    /**
     * Get the chunk file (read only), if the file exists.
     * @throw exception if the the file doesn't exists.
     * @returns the wrapper that can be used do read the file.
     */
    static std::unique_ptr<PoolChunkWrapper> get(const QString &poolPath, const QByteArray &sha256);

    /**
     * Create the chunk file (write only), if it doesn't exists.
     * @throw exception if the the file doesn't exists.
     * @returns the wrapper that can be used do write the file.
     */
    static std::unique_ptr<PoolChunkWrapper> create(const QString &poolPath, const QByteArray &sha256);

    const QByteArray &sha256() const;

    /**
     * Get the device used to fill or to read the part of the chunk
     * @returns the device.
     */
    QIODevice *getDevice();

    /**
     * Get the file directly (compressed, ...)
     * @returns the device.
     */
    QFileDevice *getFile();

    /**
     * Check that the chunk of a created device.
     */
    QByteArray checkAndClose();

private:
    PoolChunkWrapper(const QString &poolPath, const QByteArray &sha256);
    void initDevice(QFileDevice *file);

    void setSha256(const QByteArray &sha256);

    QString getChunkDir() const;
    QString getChunkPath() const;
    QString getLockPath() const;

    QDir m_poolPath;
    QByteArray m_sha256;
    QString m_sha256Str;
    QFileDevice *m_file; // Not an unique_ptr because the device will be the owner of the file and will destroy it
    std::unique_ptr<QIODevice> m_device;

    QLockFile m_lockfile;
};
