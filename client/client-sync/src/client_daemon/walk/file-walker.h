#pragma once

#include <functional>
#include <QRegExp>
#include <QString>
#include <QList>

#include <interface/file-manifest.h>

class FileWorker
{
public:
    FileWorker(const QString &shareName);

    void walk(const QString &dir, const QList<QRegExp> &includes, const QList<QRegExp> &excludes, std::function<void(const Common::FileManifest &)> process);

    const QList<std::exception> &errors();

private:
    QString m_shareName;
    QList<std::exception> m_errors;
};
