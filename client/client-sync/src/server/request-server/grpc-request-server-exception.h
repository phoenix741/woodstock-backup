#pragma once

#include <stdexcept>

class GrpcRequestServerException : public std::runtime_error
{
public:
    GrpcRequestServerException(const std::string &what_arg) : std::runtime_error(what_arg) {}
    GrpcRequestServerException(const char *what_arg) : std::runtime_error(what_arg) {}
};