#include "sha256.h"

#include <stdexcept>
#include <QFile>
#include <QCryptographicHash>
#include <QDebug>

constexpr const std::size_t BUFFER_SIZE{1 << 17}; // Should be a multiple of BUFFER_SIZE

filehash sha256(const QString &path)
{
    QFile file(path);
    if (!file.open(QFile::ReadOnly))
    {
        throw std::runtime_error("Can't open the file: " + path.toStdString()); // FIXME
    }

    filehash fhash;
    QCryptographicHash hash(QCryptographicHash::Sha3_256);
    QCryptographicHash chunks(QCryptographicHash::Sha3_256);

    char buffer[BUFFER_SIZE];
    int bufferLength = 0;

    while (!file.atEnd())
    {
        auto gcount = file.read(buffer, BUFFER_SIZE);
        hash.addData(buffer, gcount);
        chunks.addData(buffer, gcount);

        bufferLength += gcount;
        if (bufferLength >= CHUNK_SIZE)
        {
            if (bufferLength > CHUNK_SIZE)
            {
                qCritical() << "Buffer length " << bufferLength << " don't respect chunk size " << CHUNK_SIZE;
            }

            fhash.chunks.push_back(chunks.result());

            // qDebug() << "1" << bufferLength << "/" << CHUNK_SIZE << chunks.result().toHex() << file.pos() << "/" << file.size();
            chunks.reset();
            bufferLength = 0;
        }
    }

    // qDebug() << "1" << bufferLength << "/" << CHUNK_SIZE << chunks.result().toHex() << file.pos() << "/" << file.size();
    fhash.chunks.push_back(chunks.result());
    fhash.shasum = hash.result();

    file.close();

    return fhash;
}
