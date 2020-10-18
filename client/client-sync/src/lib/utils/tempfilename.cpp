#include "tempfilename.h"

#include <string>
#include <random>

QString getTemporyFileName(int length)
{
    std::string stringBuffer = "";
    stringBuffer.reserve(length); // preallocate storage

    static const std::string str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    static std::random_device rd;
    static std::mt19937 generator(rd());
    static std::uniform_int_distribution<int> dist(0, str.size() - 1);

    for (int i = 0; i < length; ++i)
        stringBuffer += str[dist(generator)];

    return QString::fromStdString(stringBuffer);
}