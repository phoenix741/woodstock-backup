#include <string>
#include <QString>
#include <QFile>

std::string readKeycert(const std::string &filename)
{
    QFile file(QString::fromStdString(filename));
    if (!file.open(QFile::ReadOnly))
    {
        throw std::runtime_error("Can't open filename " + filename);
    }
    return file.readAll().toStdString();
}
