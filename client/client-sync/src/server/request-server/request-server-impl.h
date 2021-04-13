#pragma once

#ifdef WITH_PROTOCOL_GRPC
#include "./grpc-request-server.h"

std::unique_ptr<RequestServer> getRequestServer(const QString &host)
{
    return GrpcRequestServer::create(host);
}
#endif

#ifdef WITH_PROTOCOL_HTTP2
#include "./http2-request-server.h"

std::unique_ptr<RequestServer> getRequestServer(const QString &host)
{
    return Http2RequestServer::create(host);
}
#endif