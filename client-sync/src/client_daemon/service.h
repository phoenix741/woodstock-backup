#pragma once

#include <QtService>

#include "woodstockclient.h"

class ClientService : public QtService::Service
{
    Q_OBJECT
public:
    explicit ClientService(int &argc, char **argv);

protected:
    virtual ClientService::CommandResult onStart() override;
    virtual ClientService::CommandResult onStop(int &exitCode) override;

private:
    std::unique_ptr<WoodstockClient> m_client;
};
