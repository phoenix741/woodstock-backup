#pragma once

#include <QElapsedTimer>
#include <interface/configuration.h>

class WoodstockServer
{
public:
    explicit WoodstockServer();
    virtual ~WoodstockServer();

    void start();

    Common::Configuration getConfiguration(const QString &host) const;

private:
    void printStats();

    qint64 m_lastElapsed;
    QElapsedTimer m_timer;
    qint64 nbFileRead, totalSize, transferredSize, compressedPoolSize;
};
