#include "group_grpc.h"
#include <google/protobuf/message_lite.h>

bool writeDelimitedTo(QIODevice *output, const google::protobuf::MessageLite &message)
{
    const uint32_t size = message.ByteSizeLong();
    auto bytesWritten = output->write(reinterpret_cast<const char *>(&size), sizeof(size));
    if (bytesWritten != sizeof(size))
    {
        throw std::runtime_error("Can't write message to journal");
    }

    std::string buffer;
    message.SerializeToString(&buffer);
    auto bufferWritten = output->write(buffer.c_str(), buffer.length());
    if (bufferWritten != buffer.length())
    {
        throw std::runtime_error("Can't write message to journal");
    }

    return true;
}

bool readDelimitedFrom(QIODevice *input, google::protobuf::MessageLite *message)
{
    uint32_t size;
    if (input->read(reinterpret_cast<char *>(&size), sizeof(size)) <= 0)
        return false;

    std::unique_ptr<char[]> buffer(new char[size]);
    if (input->read(buffer.get(), size) <= 0)
    {
        return false;
    };

    message->ParseFromArray(buffer.get(), size);

    return true;
}