#pragma once

#include <memory>
#include <QIODevice>
#include <zlib.h>

const auto ZLIB_CHUNK_SIZE{1 << 17};

class ZLibDevice : public QIODevice
{
    Q_OBJECT
public:
    ZLibDevice(QIODevice *deviceToUse, QObject *parent = 0);
    virtual ~ZLibDevice();
    virtual bool open(OpenMode mode);
    virtual void close();
    virtual bool isSequential() const;
    virtual qint64 size() const;

protected:
    virtual qint64 readData(char *data, qint64 maxSize);
    virtual qint64 writeData(const char *data, qint64 maxSize);

private:
    qint64 m_size;
    QByteArray m_readedData;
    unsigned char buffer_in[ZLIB_CHUNK_SIZE];
    unsigned char buffer_out[ZLIB_CHUNK_SIZE];

    std::unique_ptr<QIODevice> underlyingDevice;
    std::unique_ptr<z_stream> strm;
    Q_DISABLE_COPY(ZLibDevice)
};
