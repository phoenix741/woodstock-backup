#pragma once

#include <memory>
#include <QIODevice>
#include <QCryptographicHash>

class Sha256Device : public QIODevice
{
    Q_OBJECT
public:
    Sha256Device(QIODevice *deviceToUse, QObject *parent = 0);
    virtual ~Sha256Device();
    virtual bool open(OpenMode mode) override;
    virtual void close() override;

    virtual bool isSequential() const override;
    virtual qint64 pos() const override;
    virtual qint64 size() const override;
    virtual bool seek(qint64 pos) override;
    virtual bool atEnd() const override;

    QByteArray getHash() const;
    const qint64 dataLength() const { return m_dataLength; };

protected:
    virtual qint64 readData(char *data, qint64 maxSize) override;
    virtual qint64 writeData(const char *data, qint64 maxSize) override;

private:
    std::unique_ptr<QIODevice> underlyingDevice;
    QCryptographicHash m_hash;
    qint64 m_dataLength;

    Q_DISABLE_COPY(Sha256Device)
};
