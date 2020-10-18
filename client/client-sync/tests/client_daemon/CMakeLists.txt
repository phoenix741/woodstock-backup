include(CTest)
enable_testing(true)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 20)

find_package(ZLIB REQUIRED)
find_package(protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(Threads)
find_package(OpenSSL REQUIRED)
find_package(Qt5 COMPONENTS Core Test REQUIRED)

include_directories(${Protobuf_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories(../../src/lib)
include_directories(${CMAKE_CURRENT_BINARY_DIR}/../../src/lib)

SET(CLIENT_BACKUP_DIRECTORY "/var/lib/woodstock/client/")
add_definitions( -DCLIENT_BACKUP_DIRECTORY=${CLIENT_BACKUP_DIRECTORY} )

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/../../src/client_daemon/path_config.h.in ${CMAKE_BINARY_DIR}/generated/path_config.h)
include_directories(${CMAKE_BINARY_DIR}/generated/)

file(GLOB_RECURSE SRC CONFIGURE_DEPENDS "*.cpp" "../../src/client_daemon/walk/*.cpp")

# Create the executable

add_executable(client-service-test ${PROTO} ${SRC})
add_test(NAME client-service-test COMMAND client-service-test WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../../")
target_link_libraries(client-service-test PRIVATE woodstock Qt5::Core Qt5::Test gRPC::grpc++_reflection protobuf::libprotobuf-lite ${OPENSSL_CRYPTO_LIBRARY} ZLIB::ZLIB)
