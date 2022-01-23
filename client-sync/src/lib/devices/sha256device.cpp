#include <QDebug>
#include <QSaveFile>
#include "sha256device.h"

Sha256Device::Sha256Device(QIODevice *deviceToUse, QObject *parent) : QIODevice(parent), underlyingDevice(deviceToUse), m_hash(QCryptographicHash::Sha3_256), m_dataLength(0)
{
}

Sha256Device::~Sha256Device()
{
}

bool Sha256Device::isSequential() const
{
    return underlyingDevice->isSequential();
}

bool Sha256Device::seek(qint64 pos)
{
    m_hash.reset();
    m_dataLength = 0;
    if (!underlyingDevice->seek(pos) || !QIODevice::seek(pos))
    {
        return false;
    }
    return true;
}

qint64 Sha256Device::pos() const
{
    return QIODevice::pos();
}

qint64 Sha256Device::size() const
{
    return underlyingDevice->size();
}

bool Sha256Device::atEnd() const
{
    return QIODevice::atEnd();
}

bool Sha256Device::open(OpenMode mode)
{
    bool underlyingOk;
    if (underlyingDevice->isOpen())
    {
        underlyingOk = (underlyingDevice->openMode() == mode);
    }
    else
    {
        underlyingOk = underlyingDevice->open(mode);
    }
    if (underlyingOk)
    {
        setOpenMode(mode);
        return true;
    }

    return false;
}

void Sha256Device::close()
{
    if (QSaveFile *v = dynamic_cast<QSaveFile *>(underlyingDevice.get()))
    {
        v->commit();
    }
    else
    {
        underlyingDevice->close();
    }
    setOpenMode(NotOpen);
}

qint64 Sha256Device::readData(char *data, qint64 maxSize)
{
    auto result = underlyingDevice->read(data, maxSize);
    if (result > 0)
    {
        m_hash.addData(data, result);
        m_dataLength += result;
    }
    return result;
}

qint64 Sha256Device::writeData(const char *data, qint64 maxSize)
{
    if (maxSize > 0)
    {
        m_hash.addData(data, maxSize);
        m_dataLength += maxSize;
    }
    return underlyingDevice->write(data, maxSize);
}

QByteArray Sha256Device::getHash() const
{
    return m_hash.result();
}
