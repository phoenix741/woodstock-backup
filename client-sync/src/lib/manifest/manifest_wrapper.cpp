#include "manifest_wrapper.h"
#include "../protobuf/group_grpc.h"

ManifestWrapper::ManifestWrapper(const QString &path) : m_path(path), m_openmode(QIODevice::NotOpen)
{
}

ManifestWrapper::~ManifestWrapper()
{
    close();
}

QFile *ManifestWrapper::open(const QIODevice::OpenMode &mode, const qint64 &pos)
{
    if (m_openmode != mode)
    {
        close();
    }

    if (!m_stream)
    {
        m_openmode = mode;
        m_stream.reset(new QFile(m_path));
        m_stream->open(mode);
    }

    if (pos >= 0)
    {
        m_stream->seek(pos);
    }
    return m_stream.get();
}

void ManifestWrapper::close()
{
    if (m_stream)
    {
        m_openmode = QFile::NotOpen;
        m_stream->close();
    }
    m_stream.reset();
}
