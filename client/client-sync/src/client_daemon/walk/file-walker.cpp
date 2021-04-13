#include "file-walker.h"

#ifdef __linux__
#include <sys/stat.h>
#include <dirent.h>
#elif _WIN32
#endif

#include <QDir>
#include <QString>
#include <QRegExp>
#include <QDebug>
#include <QDateTime>

FileWorker::FileWorker(const QString &shareName) : m_shareName(shareName)
{
}

void FileWorker::walk(const QString &path, const QList<QRegExp> &includes, const QList<QRegExp> &excludes, std::function<void(const Common::FileManifest &)> process)
{
    auto absoluteCleanPath = QDir(m_shareName).filePath(path);
    auto relativeCleanPath = path;

    QDir dir(absoluteCleanPath);
    auto entryList = dir.entryInfoList(QDir::AllEntries | QDir::NoDotAndDotDot);

    for (const auto &entry : entryList)
    {
        try
        {
            auto absoluteFilePath = entry.absoluteFilePath();
            auto relativeFilePath = QDir(relativeCleanPath).filePath(entry.fileName());

            if (includes.size())
            {
                bool accepted = false;
                for (auto regexp : includes)
                {
                    accepted |= regexp.indexIn(relativeFilePath) != -1;
                }
                if (!accepted)
                {
                    qDebug() << "don't include " << relativeFilePath;
                    continue;
                }
            }
            if (excludes.size())
            {
                bool refused = false;
                for (auto regexp : excludes)
                {
                    if (regexp.indexIn(relativeFilePath) != -1)
                    {
                        refused = true;
                        break;
                    }
                }
                if (refused)
                {
                    qDebug() << "exclude " << relativeFilePath;
                    continue;
                }
            }

            if (entry.isDir())
            {
                this->walk(relativeFilePath, includes, excludes, process);
            }

            Common::FileManifest fileManifest;
            fileManifest.setPath(absoluteFilePath);
            fileManifest.stats().setSize(entry.size());
            fileManifest.stats().setLastRead(entry.lastRead().toMSecsSinceEpoch());
            fileManifest.stats().setLastModified(entry.lastModified().toMSecsSinceEpoch());
            fileManifest.stats().setCreated(entry.created().toMSecsSinceEpoch());
            fileManifest.stats().setOwnerId(entry.ownerId());
            fileManifest.stats().setGroupId(entry.groupId());

#ifdef __linux__
            struct stat sb;
            if (lstat(absoluteFilePath.toStdString().c_str(), &sb) == -1)
            {
                qDebug() << "error " << path;
                throw std::exception(); // TODO
            }
            fileManifest.stats().setMode(sb.st_mode);
#endif

            process(fileManifest);
        }
        catch (std::exception &ex)
        {
            m_errors.push_back(ex);
        }
    }
}

const QList<std::exception> &FileWorker::errors()
{
    return m_errors;
}
