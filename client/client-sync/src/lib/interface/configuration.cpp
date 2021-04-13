#include "configuration.h"
#include "configuration_p.h"

template <typename Share>
QStringList includesToArray(const Share &share)
{
    QStringList list;
    for (auto i = 0; i < share.includes_size(); i++)
    {
        list.push_back(QString::fromStdString(share.includes(i)));
    }
    return list;
}

template <typename Share>
QStringList excludesToArray(const Share &share)
{
    QStringList list;
    for (auto i = 0; i < share.excludes_size(); i++)
    {
        list.push_back(QString::fromStdString(share.excludes(i)));
    }
    return list;
}

namespace Common
{
    /* Share */

    Share::Share() : d(new ShareData())
    {
    }

    Share::Share(const QString &name) : d(new ShareData())
    {
        setName(name);
    }

    Share::Share(const Share &other) : d(other.d)
    {
    }

    Share::~Share()
    {
    }

    Share &Share::operator=(const Share &rhs)
    {
        if (this == &rhs)
            return *this; //Protect against self-assignment
        d = rhs.d;
        return *this;
    }

    void Share::setName(const QString &value)
    {
        d->name = value;
    }

    const QString &Share::name() const
    {
        return d->name;
    }

    void Share::setIncludes(const QStringList &value)
    {
        d->includes = value;
    }

    const QStringList &Share::includes() const
    {
        return d->includes;
    }

    QStringList &Share::includes()
    {
        return d->includes;
    }

    void Share::setExcludes(const QStringList &value)
    {
        d->excludes = value;
    }

    const QStringList &Share::excludes() const
    {
        return d->excludes;
    }

    QStringList &Share::excludes()
    {
        return d->excludes;
    }

    void Share::setPathPrefix(const QString &value)
    {
        d->pathPrefix = value;
    }

    const QString &Share::pathPrefix() const
    {
        return d->pathPrefix;
    }

    Share Share::fromProtobuf(const woodstock::BackupConfiguration_Share &protobufShare)
    {
        Share share;
        share.setName(QString::fromStdString(protobufShare.name()));
        share.setIncludes(includesToArray(protobufShare));
        share.setExcludes(excludesToArray(protobufShare));
        share.setPathPrefix(QString::fromStdString(protobufShare.pathprefix()));
        return share;
    }

    void Share::toProtobuf(woodstock::BackupConfiguration_Share *protobuf) const
    {
        protobuf->set_name(this->name().toStdString());
        protobuf->set_pathprefix(this->pathPrefix().toStdString());
        for (auto include : includes())
        {
            protobuf->add_includes(include.toStdString());
        }
        for (auto exclude : excludes())
        {
            protobuf->add_excludes(exclude.toStdString());
        }
    }

    /* Task */

    Task::Task() : d(new TaskData())
    {
    }

    Task::Task(const QString &command) : d(new TaskData())
    {
        setCommand(command);
    }

    Task::Task(const QList<Common::Share> &shares) : d(new TaskData())
    {
        setShares(shares);
    }

    Task::Task(const Task &other) : d(other.d)
    {
    }

    Task::~Task()
    {
    }

    Task &Task::operator=(const Task &rhs)
    {
        if (this == &rhs)
            return *this; //Protect against self-assignment
        d = rhs.d;
        return *this;
    }

    void Task::setCommand(const QString &value)
    {
        d->command = value;
    }

    const QString &Task::command() const
    {
        return d->command;
    }

    void Task::setIncludes(const QStringList &value)
    {
        d->includes = value;
    }

    QStringList &Task::includes()
    {
        return d->includes;
    }

    const QStringList &Task::includes() const
    {
        return d->includes;
    }

    void Task::setExcludes(const QStringList &value)
    {
        d->excludes = value;
    }

    QStringList &Task::excludes()
    {
        return d->excludes;
    }

    const QStringList &Task::excludes() const
    {
        return d->excludes;
    }

    void Task::setShares(const QList<Common::Share> &value)
    {
        d->shares = value;
    }

    QList<Common::Share> &Task::shares()
    {
        return d->shares;
    }

    const QList<Common::Share> &Task::shares() const
    {
        return d->shares;
    }

    Task Task::fromProtobuf(const woodstock::BackupConfiguration_Task &protobufTask)
    {
        Task task;
        task.setCommand(QString::fromStdString(protobufTask.command()));
        task.setIncludes(includesToArray(protobufTask));
        task.setExcludes(excludesToArray(protobufTask));
        for (auto i = 0; i < protobufTask.shares_size(); i++)
        {
            task.shares().push_back(Share::fromProtobuf(protobufTask.shares(i)));
        }
        return task;
    }

    void Task::toProtobuf(woodstock::BackupConfiguration_Task *task) const
    {
        task->set_command(this->command().toStdString());
        for (auto include : includes())
        {
            task->add_includes(include.toStdString());
        }
        for (auto exclude : excludes())
        {
            task->add_excludes(exclude.toStdString());
        }
        for (auto share : shares())
        {
            auto protoShare = task->add_shares();
            share.toProtobuf(protoShare);
        }
    }

    /* Operations */

    Operations::Operations() : d(new OperationsData())
    {
    }

    Operations::Operations(const Operations &other) : d(other.d)
    {
    }

    Operations::~Operations()
    {
    }

    Operations &Operations::operator=(const Operations &rhs)
    {
        if (this == &rhs)
            return *this; //Protect against self-assignment
        d = rhs.d;
        return *this;
    }

    void Operations::setTasks(const QList<Common::Task> &value)
    {
        d->tasks = value;
    }

    const QList<Common::Task> &Operations::tasks() const
    {
        return d->tasks;
    }

    QList<Common::Task> &Operations::tasks()
    {
        return d->tasks;
    }

    void Operations::setFinalisedTasks(const QList<Common::Task> &value)
    {
        d->finalisedTasks = value;
    }

    const QList<Common::Task> &Operations::finalisedTasks() const
    {
        return d->finalisedTasks;
    }

    QList<Common::Task> &Operations::finalisedTasks()
    {
        return d->finalisedTasks;
    }

    Operations Operations::fromProtobuf(const woodstock::BackupConfiguration_Operations &protobufOperations)
    {
        Operations operations;
        for (auto i = 0; i < protobufOperations.tasks_size(); i++)
        {
            operations.tasks().push_back(Task::fromProtobuf(protobufOperations.tasks(i)));
        }
        for (auto i = 0; i < protobufOperations.finalizedtasks_size(); i++)
        {
            operations.finalisedTasks().push_back(Task::fromProtobuf(protobufOperations.tasks(i)));
        }
        return operations;
    }

    void Operations::toProtobuf(woodstock::BackupConfiguration_Operations *operations) const
    {
        for (auto task : tasks())
        {
            auto protoTask = operations->add_tasks();
            task.toProtobuf(protoTask);
        }
        for (auto task : finalisedTasks())
        {
            auto protoTask = operations->add_finalizedtasks();
            task.toProtobuf(protoTask);
        }
    }

    /* Configuration */

    Configuration::Configuration() : d(new ConfigurationData())
    {
    }

    Configuration::Configuration(const Configuration &other) : d(other.d)
    {
    }

    Configuration::~Configuration()
    {
    }

    Configuration &Configuration::operator=(const Configuration &rhs)
    {
        if (this == &rhs)
            return *this; //Protect against self-assignment
        d = rhs.d;
        return *this;
    }

    void Configuration::setOperations(const Operations &value)
    {
        d->operations = value;
    }

    const Operations &Configuration::operations() const
    {
        return d->operations;
    }

    Operations &Configuration::operations()
    {
        return d->operations;
    }

    Configuration Configuration::fromProtobuf(const woodstock::BackupConfiguration &protobufConfiguration)
    {
        Configuration configuration;
        configuration.setOperations(Operations::fromProtobuf(protobufConfiguration.operations()));
        return configuration;
    }

    void Configuration::toProtobuf(woodstock::BackupConfiguration *protobuf) const
    {
        operations().toProtobuf(protobuf->mutable_operations());
    }
} // namespace Common