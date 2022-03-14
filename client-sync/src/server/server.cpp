#include "woodstock.grpc.pb.h"
#include "woodstockserver.h"

int main(int argc, char *argv[])
{
    GOOGLE_PROTOBUF_VERIFY_VERSION;

    WoodstockServer *server = new WoodstockServer();
    server->start();

    return 0;
}
