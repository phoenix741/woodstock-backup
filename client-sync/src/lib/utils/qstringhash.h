#pragma once

#include <QtGlobal>
#include <QHash>
#include <QString>
#include <functional>

#if QT_VERSION < QT_VERSION_CHECK(5, 14, 0)
namespace std
{
    template <>
    struct hash<QString>
    {
        std::size_t operator()(const QString &s) const noexcept
        {
            return (size_t)qHash(s);
        }
    };
} // namespace std
#endif