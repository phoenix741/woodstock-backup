#include "woodstockclient.h"

#include <QRegExp>
#include <QString>
#include <QFile>
#include <QLockFile>
#include <QDebug>
#include <QLocale>
#include <iomanip>
#include <QTime>

#include "request-client/request-client-impl.h"
#include "./command.h"
#include <utils/sha256.h>
#include <devices/sha256device.h>
#include <manifest/manifest.h>
#include <manifest/indexmanifest.h>
#include <walk/file-walker.h>

class RequestClientCacheImpl : public RequestClientCache
{
private:
    Manifest m_manifest;
    QLockFile m_lockFile;

public:
    RequestClientCacheImpl(const QString &path) : m_manifest("backup", path), m_lockFile(m_manifest.lockPath())
    {
        if (!m_lockFile.tryLock())
        {
            throw std::runtime_error("Can't lock the file, already a process running");
        }
        m_manifest.deleteManifest();
    }

    virtual ~RequestClientCacheImpl()
    {
    }

    virtual void addFileManifest(const Common::FileManifest &manifest) override
    {
        m_manifest.addManifest(manifest, true);
    }

    virtual void close() override
    {
        m_lockFile.unlock();
        m_manifest.compact();
    }
};

QList<QRegExp> stringToRegexp(const QStringList &list)
{
    QList<QRegExp> result;
    for (auto str : list)
    {
        result.push_back(QRegExp(str, Qt::CaseSensitive, QRegExp::WildcardUnix));
    }
    return result;
}

WoodstockClient::WoodstockClient() : m_config(new ClientConfig()), m_server(getRequestClient(this)), m_currentBackupId(-1)
{
}

WoodstockClient::~WoodstockClient()
{
}

void WoodstockClient::listen()
{
    m_server->listen();
}

void WoodstockClient::stop()
{
    m_server->stop();
}

PrepareResult WoodstockClient::prepareBackup(const Common::Configuration &configuration, qint32 lastBackupUuid, qint32 newBackupUuid)
{
    qDebug() << QString("--> Ask for preparation of the backup %1").arg(newBackupUuid);
    m_currentConfiguration = configuration;
    m_currentBackupId = newBackupUuid;

    return PrepareResult{lastBackupUuid != m_config->lastBackupNumber()};
}

std::unique_ptr<RequestClientCache> WoodstockClient::refreshCache()
{
    qDebug() << QString("--> Ask for refresh cache");
    return std::unique_ptr<RequestClientCache>(new RequestClientCacheImpl(m_config->manifestPath()));
}

void WoodstockClient::launchBackup(qint32 backupNumber, std::function<bool(const Common::JournalEntry &)> process)
{
    qDebug() << QString("--> Ask for backup launch of %1").arg(backupNumber);
    nbFileRead = 0;
    totalSize = 0;
    transferredSize = 0;
    nbFileError = 0;

    auto machineUuid = m_config->machineId();
    Manifest manifest("backup", m_config->manifestPath());
    QLockFile lockFile(manifest.lockPath());
    m_timer.start();
    m_lastElapsed = 0;

    try
    {
        qDebug() << QString("--> Lock %1").arg(manifest.lockPath());
        if (!lockFile.tryLock())
        {
            throw std::runtime_error("Can't lock the file, already a process running");
        }

        auto index = manifest.loadIndex();
        qDebug() << "Size of index " << index->indexSize;

        auto configuration = m_currentConfiguration;

        try
        {
            for (const auto task : configuration.operations().tasks())
            {
                processTask(index.get(), &manifest, process, task);
            }
        }
        catch (std::exception &ex)
        {
            // TODO : ex
            qDebug() << "Exception " << ex.what();
            for (const auto task : configuration.operations().finalisedTasks())
            {
                processTask(index.get(), &manifest, process, task);
            }
        }

        for (QString file : index->unmarkedFile())
        {
            manifest.removePath(file);
        }

        index.reset();

        qDebug() << QString("--> Compact");
        manifest.compact();
        printStats();
    }
    catch (std::exception &err)
    {
        std::cout << "Can't create the backup: " << err.what() << std::endl;
    }
    catch (const std::string &ex)
    {
        std::cout << "Can't create the backup: " << ex << std::endl;
    }
    catch (...)
    {
        std::cout << "Can't create the backup." << std::endl;
    }
    lockFile.unlock();
}

std::unique_ptr<QIODevice> WoodstockClient::getChunk(const QString filename, qint64 pos)
{
    // qDebug() << QString("--> Get the chunk %1:%2").arg(filename).arg(pos);
    QFile *device = new QFile(filename);
    if (!device->open(QIODevice::ReadOnly))
    {
        delete device;
        throw std::runtime_error("Can't open the file: " + filename.toStdString()); // FIXME
    }

    device->seek(pos);

    return std::unique_ptr<QIODevice>(device);
}

void WoodstockClient::processTask(IndexManifest *index, Manifest *manifest, std::function<bool(const Common::JournalEntry &)> process, const Common::Task &task)
{
    processCommand(task);

    for (const auto &share : task.shares())
    {
        auto includes = task.includes() + share.includes();
        auto excludes = task.excludes() + share.excludes();

        // qDebug() << QString("--> Process task %1").arg(share.name);
        FileWorker walker(share.name());
        walker.walk(
            "",
            stringToRegexp(includes),
            stringToRegexp(excludes),
            [&](const Common::FileManifest &file) {
                // qDebug() << QString("  --> Find file %1").arg(file.path);
                nbFileRead++;
                totalSize += file.stats().size();

                auto indexEntry = index->getEntry(file.path());
                bool isAdded = !indexEntry.isValid();
                if (
                    isAdded ||
                    !Manifest::compareManifest(indexEntry.toManifest(), file))
                {
                    // qDebug() << QString("    --> File is modified %1").arg(file.path);
                    auto oldManifest = manifest->getManifest(indexEntry);

                    Common::JournalEntry entry;
                    entry.setType(isAdded ? Common::ADD : Common::MODIFY);
                    entry.setManifest(file);

                    // if (!isAdded)
                    // {
                    manifest->loadManifestChunk(entry.manifest());
                    // }

                    if (process(entry))
                    {
                        manifest->addManifest(entry.manifest(), isAdded);
                        index->mark(indexEntry);
                    }
                    else
                    {
                        qWarning() << "    --> Error";
                    }
                }

                printStats();
            });
        nbFileError += walker.errors().size();
        printStats();
    }
}

void WoodstockClient::processCommand(const Common::Task &task)
{
    if (!task.command().isEmpty())
    {
        Command command;
        command.execute(task.command());
    }
}

void WoodstockClient::printStats()
{
    if (m_timer.elapsed() - m_lastElapsed > 5000)
    {
        std::cout << "\33[2K\r[" << QTime::fromMSecsSinceStartOfDay(m_timer.elapsed()).toString().toStdString() << "]" << std::setw(9) << std::setfill(' ') << nbFileRead << " files read / " << std::setw(4) << std::setfill(' ') << nbFileError << " error(s) / " << QLocale::c().formattedDataSize(totalSize).toStdString() << " total / " << QLocale::c().formattedDataSize(transferredSize).toStdString() << " transferred" << std::flush;
        m_lastElapsed = m_timer.elapsed();
    }
}
