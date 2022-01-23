#pragma once

#include <memory>
#include <functional>
#include <QFile>
#include <QString>
#include <protobuf/group_grpc.h>

class ManifestWrapper
{
public:
    ManifestWrapper(const QString &path);
    ~ManifestWrapper();

    template <class Message>
    bool readMessage(Message *message, const qint64 &pos = 1)
    {
        auto rawStream = open(QFile::ReadOnly, pos);
        return readDelimitedFrom(rawStream, message);
    }

    template <class Message>
    void readAllMessages(std::function<void(const Message &, const qint64 &)> process)
    {
        auto rawStream = open(QFile::ReadOnly);
        Message message;
        qint64 pos = 0;
        while (readDelimitedFrom(rawStream, &message))
        {
            process(message, pos);

            message.Clear();
            pos = rawStream->pos();
        }
    }

    template <class Message>
    bool writeMessage(const Message &message)
    {
        auto rawStream = open(QFile::WriteOnly);
        return writeDelimitedTo(rawStream, message);
    }

    void close();

private:
    QFile *open(const QFile::OpenMode &mode, const qint64 &pos = -1);

    QString m_path;
    QFile::OpenMode m_openmode;
    std::unique_ptr<QFile> m_stream;
};
