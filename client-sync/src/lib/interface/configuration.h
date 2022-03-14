#pragma once

#include <QString>
#include <QList>
#include <QStringList>
#include <QSharedDataPointer>

#include "woodstock.pb.h"

namespace Common
{
    class ShareData;
    class TaskData;
    class OperationsData;
    class ConfigurationData;

    struct Share
    {
        Share();
        Share(const QString &name);
        Share(const Share &other);
        ~Share();

        Share &operator=(const Share &rhs);

        void setName(const QString &value);
        const QString &name() const;

        void setIncludes(const QStringList &value);
        const QStringList &includes() const;
        QStringList &includes();

        void setExcludes(const QStringList &value);
        const QStringList &excludes() const;
        QStringList &excludes();

        void setPathPrefix(const QString &value);
        const QString &pathPrefix() const;

        static Share fromProtobuf(const woodstock::BackupConfiguration_Share &share);
        void toProtobuf(woodstock::BackupConfiguration_Share *protobuf) const;

    private:
        QSharedDataPointer<ShareData> d;
    };

    struct Task
    {
        Task();
        Task(const QString &command);
        Task(const QList<Common::Share> &shares);
        Task(const Task &other);
        ~Task();

        Task &operator=(const Task &rhs);

        void setCommand(const QString &value);
        const QString &command() const;

        void setIncludes(const QStringList &value);
        QStringList &includes();
        const QStringList &includes() const;

        void setExcludes(const QStringList &value);
        QStringList &excludes();
        const QStringList &excludes() const;

        void setShares(const QList<Common::Share> &value);
        QList<Common::Share> &shares();
        const QList<Common::Share> &shares() const;

        static Task fromProtobuf(const woodstock::BackupConfiguration_Task &task);
        void toProtobuf(woodstock::BackupConfiguration_Task *protobuf) const;

    private:
        QSharedDataPointer<TaskData> d;
    };

    struct Operations
    {
        Operations();
        Operations(const Operations &other);
        ~Operations();

        Operations &operator=(const Operations &rhs);

        void setTasks(const QList<Common::Task> &value);
        const QList<Common::Task> &tasks() const;
        QList<Common::Task> &tasks();

        void setFinalisedTasks(const QList<Common::Task> &value);
        const QList<Common::Task> &finalisedTasks() const;
        QList<Common::Task> &finalisedTasks();

        static Operations fromProtobuf(const woodstock::BackupConfiguration_Operations &operations);
        void toProtobuf(woodstock::BackupConfiguration_Operations *protobuf) const;

    private:
        QSharedDataPointer<OperationsData> d;
    };

    struct Configuration
    {
        Configuration();
        Configuration(const Configuration &other);
        ~Configuration();

        Configuration &operator=(const Configuration &rhs);

        void setOperations(const Operations &value);
        const Operations &operations() const;
        Operations &operations();

        static Configuration fromProtobuf(const woodstock::BackupConfiguration &configuration);
        void toProtobuf(woodstock::BackupConfiguration *protobuf) const;

    private:
        QSharedDataPointer<ConfigurationData> d;
    };
} // namespace Common

Q_DECLARE_TYPEINFO(Common::Share, Q_MOVABLE_TYPE);
Q_DECLARE_TYPEINFO(Common::Task, Q_MOVABLE_TYPE);
Q_DECLARE_TYPEINFO(Common::Operations, Q_MOVABLE_TYPE);
Q_DECLARE_TYPEINFO(Common::Configuration, Q_MOVABLE_TYPE);
