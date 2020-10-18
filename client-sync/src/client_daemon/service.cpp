#include "service.h"

ClientService::ClientService(int &argc, char **argv) : Service{argc, argv}, m_client(new WoodstockClient())
{
}

ClientService::CommandResult ClientService::onStart()
{
    qDebug() << "Service was started";

    m_client->listen();

    qDebug() << "Service is listening";
    return ClientService::CommandResult::Completed; //service is now assumed started
}

ClientService::CommandResult ClientService::onStop(int &exitCode)
{
    qDebug() << "Stop received";
    
    exitCode = 0;

    m_client->stop();

    qDebug() << "Stopped";
    return ClientService::CommandResult::Completed;
}
