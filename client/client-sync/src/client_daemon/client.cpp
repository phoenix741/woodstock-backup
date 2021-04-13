#include "service.h"

int main(int argc, char *argv[], char **envp)
{
    // IMPORTANT: do NOT create a QCoreApplication here - this is done internally by the backends!
    // also, do nothing else in the main besides setting the serices properties! Any setup etc. must all be
    // done in the onStart method!!!
    ClientService service{argc, argv};
    return service.exec();
}
