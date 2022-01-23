#include "clientconfig.h"
#include "path_config.h"
#include <QDir>
#include <QDebug>

#ifdef __linux__
#include <unistd.h>
#elif _WIN32
#endif

#include <QUuid>
#include <devices/qstdstream.h>

#include "client.pb.h"

using woodstock::ClientConfiguration;

ClientConfig::ClientConfig()
{
    loadConfiguration();
    if (!m_machineId.size())
    {
        m_machineId = QUuid::createUuid().toString();
        saveConfiguration();
    }

    qInfo() << "Configration path is = " << m_manifestPath;
    qInfo() << "The machine has the UUID = " << m_machineId;
}

const QString &ClientConfig::machineId() const
{
    return m_machineId;
}

const QString &ClientConfig::manifestPath() const
{
    return m_manifestPath;
}

qint32 ClientConfig::lastBackupNumber() const
{
    return m_lastBackupNumber;
}

void ClientConfig::setLastBackupNumber(qint32 number)
{
    m_lastBackupNumber = number;
    saveConfiguration();
}

void ClientConfig::loadConfiguration()
{
    m_manifestPath = getClientBackupDirectory();

#ifdef __linux__
    auto effectuveUser = geteuid();

    if (effectuveUser)
    {
        std::cout << "Executing the application as non-root can be source of 'permission denied'." << std::endl;
        m_manifestPath = QString(getenv("HOME")) + "/.woodstock";
    }
#elif _WIN32
    // FIXME
#endif

    QFile configFile(m_manifestPath + "/config");
    try
    {
        if (!configFile.open(QIODevice::ReadOnly))
        {
            throw std::runtime_error("Can't open the config file");
        }
        QStdIStream istream(&configFile);

        ClientConfiguration clientConfiguration;
        clientConfiguration.ParseFromIstream(&istream);

        m_machineId = QString::fromStdString(clientConfiguration.machineid());
        m_lastBackupNumber = clientConfiguration.lastbackupnumber();
    }
    catch (...)
    {
        // Finally
        m_lastBackupNumber = -1;
    }
    configFile.close();
}

void ClientConfig::saveConfiguration()
{
    QDir().mkpath(m_manifestPath);
    QFile configFile(m_manifestPath + "/config");
    if (!configFile.open(QIODevice::WriteOnly))
    {
        throw std::runtime_error("Can't open the config file");
    }

    QStdOStream outfile(&configFile);
    ClientConfiguration clientConfiguration;
    clientConfiguration.set_machineid(m_machineId.toStdString());
    clientConfiguration.set_lastbackupnumber(m_lastBackupNumber);
    clientConfiguration.SerializeToOstream(&outfile);

    configFile.close();
}