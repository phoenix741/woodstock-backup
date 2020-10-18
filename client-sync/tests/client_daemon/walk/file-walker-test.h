#include <QtTest/QtTest>

class TestFileWorker : public QObject
{
    Q_OBJECT
private slots:
    void walkOnFile();
    void walkOnAllFile();
};
