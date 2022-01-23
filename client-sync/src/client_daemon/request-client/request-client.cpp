#include "request-client.h"

RequestClientCache::~RequestClientCache()
{
}

RequestClientImplementation::~RequestClientImplementation()
{
}

RequestClient::RequestClient(RequestClientImplementation *impl) : m_impl(impl)
{
}

RequestClient::~RequestClient()
{
}
