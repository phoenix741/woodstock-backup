#include <QtTest/QtTest>

class TestSha256Bench : public QObject
{
    Q_OBJECT
private slots:
    void initTestCase();

    void benchSha1();
    void benchSha512();
    void benchSha256();
    void benchSha3_512();
    void benchSha3_256();
    void benchMd5();
    void benchMd4();

private:
    std::string buffer;
};
