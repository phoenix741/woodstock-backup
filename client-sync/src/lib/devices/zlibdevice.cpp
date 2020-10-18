#include <QDebug>
#include <QByteArray>
#include <QSaveFile>
#include <stdexcept>
#include "zlibdevice.h"

ZLibDevice::ZLibDevice(QIODevice *deviceToUse, QObject *parent) : QIODevice(parent), underlyingDevice(deviceToUse), strm(new z_stream())
{
}

ZLibDevice::~ZLibDevice()
{
    close();
}

bool ZLibDevice::isSequential() const
{
    return true;
}

qint64 ZLibDevice::size() const
{
    return m_size;
}

bool ZLibDevice::open(OpenMode mode)
{
    m_size = 0;

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
        strm->zalloc = Z_NULL;
        strm->zfree = Z_NULL;
        strm->opaque = Z_NULL;
        strm->avail_in = 0;
        strm->next_in = Z_NULL;
        strm->avail_out = 0;
        strm->next_out = Z_NULL;

        int ret;
        if (mode == ReadOnly)
        {
            ret = inflateInit(strm.get());
        }
        else if (mode == WriteOnly)
        {
            ret = deflateInit(strm.get(), Z_BEST_COMPRESSION);
        }
        else
        {
            return false;
        }
        if (ret != Z_OK)
        {
            return false;
        }

        setOpenMode(mode);
        return true;
    }

    return false;
}

void ZLibDevice::close()
{
    if (this->openMode() == WriteOnly)
    {
        this->write("", 0);
        deflateEnd(strm.get());
    }
    else
    {
        inflateEnd(strm.get());
    }

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

qint64 ZLibDevice::readData(char *data, qint64 maxSize)
{
    QByteArray readedData = m_readedData;
    int ret;

    if (readedData.size() < maxSize)
    {
        do
        {
            if (strm->avail_in <= 0)
            {
                strm->avail_in = underlyingDevice->read((char *)(buffer_in), ZLIB_CHUNK_SIZE);
                strm->next_in = buffer_in;
            }
            if (strm->avail_in == 0)
            {
                break;
            }

            strm->avail_out = ZLIB_CHUNK_SIZE;
            strm->next_out = buffer_out;

            ret = inflate(strm.get(), Z_NO_FLUSH);
            assert(ret != Z_STREAM_ERROR);
            switch (ret)
            {
            case Z_NEED_DICT:
                ret = Z_DATA_ERROR; /* and fall through */
            case Z_DATA_ERROR:
            case Z_MEM_ERROR:
                throw std::runtime_error(QString("Stream infrate %1").arg(ret).toStdString());
            }

            auto have = ZLIB_CHUNK_SIZE - strm->avail_out;
            readedData.append((char *)buffer_out, have);
        } while (readedData.size() < maxSize && ret != Z_STREAM_END);
    }

    auto returnSize = qMin((qint64)readedData.size(), maxSize);
    memcpy(data, readedData.data(), returnSize);
    m_readedData = readedData.right(readedData.size() - maxSize);

    m_size += returnSize;
    return returnSize;
}

qint64 ZLibDevice::writeData(const char *data, qint64 maxSize)
{
    strm->next_in = (unsigned char *)data;
    strm->avail_in = maxSize;
    auto eof = maxSize == 0 ? Z_FINISH : Z_NO_FLUSH;
    auto total = 0;

    m_size += maxSize;

    do
    {

        strm->avail_out = ZLIB_CHUNK_SIZE;
        strm->next_out = buffer_out;

        auto ret = deflate(strm.get(), eof); /* no bad return value */
        assert(ret != Z_STREAM_ERROR);       /* state not clobbered */

        auto have = ZLIB_CHUNK_SIZE - strm->avail_out;

        auto writtenBytes = underlyingDevice->write((char *)(buffer_out), have);
        total += writtenBytes;
        if (writtenBytes != have)
        {
            return total;
        }
    } while (strm->avail_out == 0);
    assert(strm->avail_in == 0); /* all input will be used */

    return total;
}