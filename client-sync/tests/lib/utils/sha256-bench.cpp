#include "sha256-bench.h"

#include <QCryptographicHash>

constexpr const std::size_t BUFFER_SIZE{1 << 12}; // Should be a multiple of BUFFER_SIZE

std::string getBuffer(int length)
{
    std::string stringBuffer = "";
    stringBuffer.reserve(length); // preallocate storage

    static const std::string str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    static std::random_device rd;
    static std::mt19937 generator(rd());
    static std::uniform_int_distribution<int> dist(0, str.size() - 1);

    for (int i = 0; i < length; ++i)
        stringBuffer += str[dist(generator)];

    return stringBuffer;
}

void TestSha256Bench::initTestCase()
{
    buffer = getBuffer(BUFFER_SIZE);
}

void TestSha256Bench::benchSha1()
{
    QCryptographicHash hash(QCryptographicHash::Sha1);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchSha512()
{
    QCryptographicHash hash(QCryptographicHash::Sha512);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchSha256()
{
    QCryptographicHash hash(QCryptographicHash::Sha256);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchSha3_512()
{
    QCryptographicHash hash(QCryptographicHash::Sha3_512);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchSha3_256()
{
    QCryptographicHash hash(QCryptographicHash::Sha3_256);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchMd5()
{
    QCryptographicHash hash(QCryptographicHash::Md5);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

void TestSha256Bench::benchMd4()
{
    QCryptographicHash hash(QCryptographicHash::Md4);

    QBENCHMARK
    {
        hash.addData(buffer.c_str(), buffer.size());
        hash.result();
    }
}

QTEST_MAIN(TestSha256Bench)
