#pragma once

#include <memory>

#include "request-client.h"

#include <nghttp2/asio_http2_server.h>

using namespace nghttp2::asio_http2;
using namespace nghttp2::asio_http2::server;

class Http2RequestClient : public RequestClient
{
public:
    Http2RequestClient(RequestClientImplementation *impl);
    virtual ~Http2RequestClient();

    virtual void listen() override;
    virtual void stop() override;

    static std::unique_ptr<Http2RequestClient> create(RequestClientImplementation *impl);

private:
    std::unique_ptr<boost::system::error_code> ec;
    std::unique_ptr<boost::asio::ssl::context> tls;
    std::unique_ptr<http2> server;
};
