#pragma once

#ifdef WITH_PROTOCOL_GRPC
#include "./grpc-request-client.h"

std::unique_ptr<RequestClient> getRequestClient(RequestClientImplementation *impl)
{
    return GrpcRequestClient::create(impl);
}
#endif

#ifdef WITH_PROTOCOL_HTTP2
#include "./http2-request-client.h"

std::unique_ptr<RequestClient> getRequestClient(RequestClientImplementation *impl)
{
    return Http2RequestClient::create(impl);
}
#endif