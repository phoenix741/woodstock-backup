#pragma once

#include <QString>

class ClientConfig
{
public:
    explicit ClientConfig();

    const QString &machineId() const;
    const QString &manifestPath() const;

    qint32 lastBackupNumber() const;
    void setLastBackupNumber(qint32 number);

private:
    void loadConfiguration();
    void saveConfiguration();

    QString m_machineId;
    QString m_manifestPath;
    qint32 m_lastBackupNumber;
};
