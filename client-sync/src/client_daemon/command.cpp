#include "command.h"

#include <QProcess>
#include <QDebug>

Command::Command(QObject *parent) : QObject(parent), m_process(new QProcess(this))
{
    connect(m_process, SIGNAL(readyReadStandardOutput()), this, SLOT(processOutputStandard()));
    connect(m_process, SIGNAL(readyReadStandardError()), this, SLOT(processOutputError()));
}

Command::~Command()
{
}

bool Command::execute(const QString &command)
{
    m_process->start(command, QStringList());
    if (!m_process->waitForStarted())
    {
        return false;
    }

    if (!m_process->waitForFinished())
    {
        return false;
    }

    return true;
}

void Command::processOutputStandard()
{
    QByteArray output = m_process->readAllStandardOutput();
    qDebug() << output; // FIXME: Send log
}

void Command::processOutputError()
{
    QByteArray output = m_process->readAllStandardError();
    qWarning() << output; // FIXME: Send log
}
