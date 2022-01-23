#pragma once

#include <QObject>

class QProcess;

class Command : public QObject
{
    Q_OBJECT
public:
    Command(QObject *parent = 0);
    virtual ~Command();

    bool execute(const QString &command);
private slots:
    void processOutputStandard();
    void processOutputError();

private:
    QProcess *m_process;
};
