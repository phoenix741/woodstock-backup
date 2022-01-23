#pragma once

#include <stdexcept>

class GrpcRequestClientException : public std::runtime_error
{
public:
    GrpcRequestClientException(const std::string &what_arg) : std::runtime_error(what_arg) {}
    GrpcRequestClientException(const char *what_arg) : std::runtime_error(what_arg) {}
};