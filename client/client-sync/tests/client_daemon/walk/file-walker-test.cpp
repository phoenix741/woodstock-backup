#include "file-walker-test.h"
#include "../../../src/client_daemon/walk/file-walker.h"

#include <QDir>
#include <QRegExp>

void TestFileWorker::walkOnFile()
{
    auto dir = QDir(QDir::currentPath());
    FileWorker walker(dir.absolutePath());

    QStringList filenames;
    walker.walk(
        "tests",
        {},
        {},
        [&](const Common::FileManifest &file) {
            filenames.append(file.path());
        });

    QStringList mappedList;
    std::transform(filenames.begin(),
                   filenames.end(),
                   std::back_inserter(mappedList),
                   [=](const QString &path) { return path.midRef(dir.absolutePath().size()).toString(); });

    QCOMPARE(walker.errors().length(), 0);
    QCOMPARE(mappedList, QStringList({"/tests/client_daemon/CMakeLists.txt", "/tests/client_daemon/walk/file-walker-test.cpp", "/tests/client_daemon/walk/file-walker-test.h", "/tests/client_daemon/walk", "/tests/client_daemon", "/tests/CMakeLists.txt"}));
}

void TestFileWorker::walkOnAllFile()
{
    auto dir = QDir(QDir::currentPath());
    FileWorker walker(dir.absolutePath());

    QStringList filenames;
    walker.walk(
        "",
        {},
        {QRegExp("/.git"),
         QRegExp("/.vscode"),
         QRegExp("/build"),
         QRegExp("/vcpkg"),
         QRegExp("/GPATH"),
         QRegExp("/GRTAGS"),
         QRegExp("/GTAGS")},
        [&](const Common::FileManifest &file) {
            filenames.append(file.path());
        });

    QStringList mappedList;
    std::transform(filenames.begin(),
                   filenames.end(),
                   std::back_inserter(mappedList),
                   [=](const QString &path) { return path.midRef(dir.absolutePath().size()).toString(); });

    QCOMPARE(walker.errors().length(), 0);
    QCOMPARE(mappedList, QStringList({"/3rd_party/CMakeLists.txt", "/3rd_party", "/CMakeLists.txt", "/Dockerfile", "/README.md", "/src/client_daemon/client.cpp", "/src/client_daemon/client.proto", "/src/client_daemon/clientconfig.cpp", "/src/client_daemon/clientconfig.h", "/src/client_daemon/CMakeLists.txt", "/src/client_daemon/command.cpp", "/src/client_daemon/command.h", "/src/client_daemon/path_config.h.in", "/src/client_daemon/request-client/grpc-request-client-exception.h", "/src/client_daemon/request-client/grpc-request-client.cpp", "/src/client_daemon/request-client/grpc-request-client.h", "/src/client_daemon/request-client/request-client.cpp", "/src/client_daemon/request-client/request-client.h", "/src/client_daemon/request-client", "/src/client_daemon/service.cpp", "/src/client_daemon/service.h", "/src/client_daemon/state.plantuml", "/src/client_daemon/walk/file-walker.cpp", "/src/client_daemon/walk/file-walker.h", "/src/client_daemon/walk", "/src/client_daemon/woodstockclient.cpp", "/src/client_daemon/woodstockclient.h", "/src/client_daemon", "/src/CMakeLists.txt", "/src/lib/CMakeLists.txt", "/src/lib/devices/qstdstream.cpp", "/src/lib/devices/qstdstream.h", "/src/lib/devices/sha256device.cpp", "/src/lib/devices/sha256device.h", "/src/lib/devices/zlibdevice.cpp", "/src/lib/devices/zlibdevice.h", "/src/lib/devices", "/src/lib/interface/configuration.cpp", "/src/lib/interface/configuration.h", "/src/lib/interface/file-manifest.cpp", "/src/lib/interface/file-manifest.h", "/src/lib/interface", "/src/lib/manifest/indexmanifest.cpp", "/src/lib/manifest/indexmanifest.h", "/src/lib/manifest/manifest.cpp", "/src/lib/manifest/manifest.h", "/src/lib/manifest/manifest_wrapper.cpp", "/src/lib/manifest/manifest_wrapper.h", "/src/lib/manifest", "/src/lib/pool/pool-chunk-refcnt.cpp", "/src/lib/pool/pool-chunk-refcnt.h", "/src/lib/pool/pool-chunk-wrapper.cpp", "/src/lib/pool/pool-chunk-wrapper.h", "/src/lib/pool/pool.cpp", "/src/lib/pool/pool.h", "/src/lib/pool", "/src/lib/protobuf/group_grpc.cpp", "/src/lib/protobuf/group_grpc.h", "/src/lib/protobuf", "/src/lib/server", "/src/lib/utils/qstringhash.h", "/src/lib/utils/sha256.cpp", "/src/lib/utils/sha256.h", "/src/lib/utils", "/src/lib/woodstock.proto", "/src/lib", "/src/server/CMakeLists.txt", "/src/server/request-server/grpc-request-server-exception.h", "/src/server/request-server/grpc-request-server.cpp", "/src/server/request-server/grpc-request-server.h", "/src/server/request-server/request-server.cpp", "/src/server/request-server/request-server.h", "/src/server/request-server", "/src/server/server.cpp", "/src/server/woodstockserver.cpp", "/src/server/woodstockserver.h", "/src/server", "/src/tools/CMakeLists.txt", "/src/tools/readindex/CMakeLists.txt", "/src/tools/readindex/readindex.cpp", "/src/tools/readindex", "/src/tools", "/src", "/tests/client_daemon/CMakeLists.txt", "/tests/client_daemon/walk/file-walker-test.cpp", "/tests/client_daemon/walk/file-walker-test.h", "/tests/client_daemon/walk", "/tests/client_daemon", "/tests/CMakeLists.txt", "/tests"}));
}

QTEST_MAIN(TestFileWorker)
