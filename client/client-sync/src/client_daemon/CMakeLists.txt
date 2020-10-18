set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 20)

find_package(ZLIB REQUIRED)
find_package(protobuf CONFIG REQUIRED)
find_package(Threads)
find_package(OpenSSL REQUIRED)
find_package(Qt5 COMPONENTS Core Network Service REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(TCMALLOC REQUIRED IMPORTED_TARGET libtcmalloc_minimal)

if(WITH_PROTOCOL_GRPC)
find_package(gRPC CONFIG REQUIRED)
endif()

if(WITH_PROTOCOL_HTTP2)
find_package(Boost COMPONENTS system REQUIRED)
pkg_check_modules(NGHTTP2_ASIO REQUIRED IMPORTED_TARGET libnghttp2_asio)
pkg_check_modules(NGHTTP2 REQUIRED IMPORTED_TARGET libnghttp2)
endif()

include_directories(${Protobuf_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories(../lib)
include_directories(${CMAKE_CURRENT_BINARY_DIR}/../lib)
include_directories(${TCMALLOC_INCLUDE_DIRS})
include_directories(${NGHTTP2_INCLUDE_DIRS})
include_directories(${NGHTTP2_ASIO_INCLUDE_DIRS})

IF (NOT DEFINED CLIENT_BACKUP_DIRECTORY)
    SET(CLIENT_BACKUP_DIRECTORY "/var/lib/woodstock/client/")
ENDIF()
add_definitions(-DCLIENT_BACKUP_DIRECTORY=${CLIENT_BACKUP_DIRECTORY} )

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/path_config.h.in ${CMAKE_BINARY_DIR}/generated/path_config.h)
include_directories(${CMAKE_BINARY_DIR}/generated/)

file(GLOB_RECURSE SRC CONFIGURE_DEPENDS "*.cpp")
set(PROTO ${CMAKE_CURRENT_SOURCE_DIR}/client.proto)

# Create the executable

add_executable(client ${PROTO} ${SRC})
target_link_libraries(client PRIVATE woodstock Qt5::Core Qt5::Network Qt5::Service protobuf::libprotobuf-lite ${OPENSSL_CRYPTO_LIBRARY} ZLIB::ZLIB PkgConfig::TCMALLOC ${PROTOCOL_LIBRARY}) 
protobuf_generate(TARGET client LANGUAGE cpp)
 