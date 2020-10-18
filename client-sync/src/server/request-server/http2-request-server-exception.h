#pragma once

#include <stdexcept>

class Http2RequestServerException : public std::runtime_error
{
public:
    Http2RequestServerException(const std::string &what_arg) : std::runtime_error(what_arg) {}
    Http2RequestServerException(const char *what_arg) : std::runtime_error(what_arg) {}
};