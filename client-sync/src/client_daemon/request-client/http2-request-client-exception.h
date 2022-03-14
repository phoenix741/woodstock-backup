#pragma once

#include <stdexcept>

class Http2RequestClientException : public std::runtime_error
{
public:
    Http2RequestClientException(const std::string &what_arg) : std::runtime_error(what_arg) {}
    Http2RequestClientException(const char *what_arg) : std::runtime_error(what_arg) {}
};