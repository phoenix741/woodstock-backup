#pragma once

#include "configuration.h"

namespace Common
{
    struct ShareData : public QSharedData
    {
        ShareData() {}
        ShareData(const ShareData &other)
            : QSharedData(other),
              name(other.name),
              includes(other.includes),
              excludes(other.excludes),
              pathPrefix(other.pathPrefix) {}
        ~ShareData() {}

        QString name;
        QStringList includes;
        QStringList excludes;
        QString pathPrefix;
    };

    struct TaskData : public QSharedData
    {
        TaskData() {}
        TaskData(const TaskData &other)
            : QSharedData(other),
              command(other.command),
              includes(other.includes),
              excludes(other.excludes),
              shares(other.shares) {}
        ~TaskData() {}

        QString command;
        QStringList includes;
        QStringList excludes;
        QList<Common::Share> shares;
    };

    struct OperationsData : public QSharedData
    {
        OperationsData() {}
        OperationsData(const OperationsData &other)
            : QSharedData(other),
              tasks(other.tasks),
              finalisedTasks(other.finalisedTasks) {}
        ~OperationsData() {}

        QList<Common::Task> tasks;
        QList<Common::Task> finalisedTasks;
    };

    struct ConfigurationData : public QSharedData
    {
        ConfigurationData() {}
        ConfigurationData(const ConfigurationData &other)
            : QSharedData(other),
              operations(other.operations) {}
        ~ConfigurationData() {}

        Operations operations;
    };
} // namespace Common