#pragma once

#include <QIODevice>

namespace google
{
    namespace protobuf
    {
        class MessageLite;
    } // namespace protobuf
} // namespace google

bool writeDelimitedTo(QIODevice *rawOutput, const google::protobuf::MessageLite &message);

bool readDelimitedFrom(QIODevice *rawInput, google::protobuf::MessageLite *message);
