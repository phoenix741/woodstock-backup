#pragma once

#include <QString>
#include <QByteArray>
#include <QList>

const int CHUNK_SIZE = 1 << 22;

struct filehash
{
    QByteArray shasum;
    QList<QByteArray> chunks;
};

filehash sha256(const QString &path);
