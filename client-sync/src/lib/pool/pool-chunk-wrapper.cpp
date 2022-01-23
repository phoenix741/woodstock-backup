#include "pool-chunk-wrapper.h"

#include <stdexcept>
#include <QDebug>
#include <QSaveFile>
#include "../devices/zlibdevice.h"
#include "../devices/sha256device.h"
#include "../utils/sha256.h"
#include "../utils/tempfilename.h"

PoolChunkWrapper::PoolChunkWrapper(const QString &poolPath, const QByteArray &sha256) : m_poolPath(poolPath), m_sha256(sha256), m_sha256Str(sha256.toHex()), m_lockfile(getLockPath())
{
    QDir(getChunkDir()).mkpath(".");
}

PoolChunkWrapper::~PoolChunkWrapper()
{
    if (m_lockfile.isLocked())
    {
        m_lockfile.unlock();
    }
}

void PoolChunkWrapper::initDevice(QFileDevice *file)
{
    Q_ASSERT_X(file != NULL, "m_file", "m_file should be not null");

    m_file = file;
    ZLibDevice *zlibDevice = new ZLibDevice(m_file);
    Sha256Device *sha256Device = new Sha256Device(zlibDevice);
    m_device = std::unique_ptr<Sha256Device>(sha256Device);
}

bool PoolChunkWrapper::exists(const QString &poolPath, const QByteArray &sha256)
{
    PoolChunkWrapper wrapper(poolPath, sha256);
    return QFile::exists(wrapper.getChunkPath());
}

std::unique_ptr<PoolChunkWrapper> PoolChunkWrapper::get(const QString &poolPath, const QByteArray &sha256)
{
    auto wrapper = new PoolChunkWrapper(poolPath, sha256);
    wrapper->initDevice(new QFile(wrapper->getChunkPath()));

    if (!wrapper->m_device->open(QFile::ReadOnly))
    {
        throw std::runtime_error("Can't open the file " + wrapper->getChunkPath().toStdString() + " for reading.");
    }

    return std::unique_ptr<PoolChunkWrapper>(wrapper);
}

std::unique_ptr<PoolChunkWrapper> PoolChunkWrapper::create(const QString &poolPath, const QByteArray &sha256)
{
    auto wrapper = new PoolChunkWrapper(poolPath, sha256);

    wrapper->m_lockfile.lock();

    wrapper->initDevice(new QSaveFile(wrapper->getChunkPath()));
    if (!wrapper->m_device->open(QFile::WriteOnly))
    {
        throw std::runtime_error("Can't open the file " + wrapper->getChunkPath().toStdString() + " for writing.");
    }

    return std::unique_ptr<PoolChunkWrapper>(wrapper);
}

QIODevice *PoolChunkWrapper::getDevice()
{
    return m_device.get();
}

QFileDevice *PoolChunkWrapper::getFile()
{
    return m_file;
}

void PoolChunkWrapper::setSha256(const QByteArray &sha256)
{
    m_sha256 = sha256;
    m_sha256Str = sha256.toHex();
    if (QSaveFile *v = dynamic_cast<QSaveFile *>(m_file))
    {
        v->setFileName(getChunkPath());
    }
    else if (QFile *v = dynamic_cast<QFile *>(m_file))
    {
        v->setFileName(getChunkPath());
    }
}

const QByteArray &PoolChunkWrapper::sha256() const
{
    return m_sha256;
}

QByteArray PoolChunkWrapper::checkAndClose()
{
    try
    {
        auto sha256OfDevice = dynamic_cast<Sha256Device *>(m_device.get())->getHash();
        if (dynamic_cast<Sha256Device *>(m_device.get())->dataLength() > CHUNK_SIZE)
        {
            qCritical() << "Chunk length " << dynamic_cast<Sha256Device *>(m_device.get())->dataLength() << " don't respect chunk size " << CHUNK_SIZE;
        }

        if (m_sha256 != sha256OfDevice || m_sha256.isEmpty())
        {
            qWarning() << "error check sum " << m_sha256Str << " != " << sha256OfDevice.toHex();
            //setSha256(sha256OfDevice);
            throw std::runtime_error("Invalid chunk " + sha256OfDevice.toHex().toStdString()); // FIXME: Log
        }

        m_device->close();
        m_lockfile.unlock();

        return sha256OfDevice;
    }
    catch (...)
    {
        if (QSaveFile *v = dynamic_cast<QSaveFile *>(m_file))
        {
            v->cancelWriting();
        }
        m_lockfile.unlock();

        auto err = std::current_exception();
        std::rethrow_exception(err);
    }
}

QString PoolChunkWrapper::getChunkDir() const
{
    if (!m_sha256Str.isEmpty())
    {
        QStringRef part1 = m_sha256Str.midRef(0, 2);
        QStringRef part2 = m_sha256Str.midRef(2, 2);
        QStringRef part3 = m_sha256Str.midRef(4, 2);

        return m_poolPath.absoluteFilePath(part1 + QDir::separator() + part2 + QDir::separator() + part3);
    }
    else
    {
        return m_poolPath.absoluteFilePath("_new");
    }
}

QString PoolChunkWrapper::getChunkPath() const
{
    return QDir(getChunkDir()).absoluteFilePath((m_sha256Str.isEmpty() ? getTemporyFileName(32) : m_sha256Str) + "-sha256.zz");
}

QString PoolChunkWrapper::getLockPath() const
{
    return QDir(getChunkDir()).absoluteFilePath("LOCK");
}