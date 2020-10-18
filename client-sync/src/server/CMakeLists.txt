set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 20)

find_package(ZLIB REQUIRED)
find_package(protobuf CONFIG REQUIRED)
find_package(Threads)
find_package(OpenSSL REQUIRED)
find_package(Qt5 COMPONENTS Core REQUIRED)
find_package(PkgConfig REQUIRED)
find_package(rxcpp CONFIG REQUIRED)
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

IF (NOT DEFINED SERVER_BACKUP_DIRECTORY)
    SET(SERVER_BACKUP_DIRECTORY "/var/lib/woodstock/")
ENDIF()
add_definitions( -DSERVER_BACKUP_DIRECTORY=${SERVER_BACKUP_DIRECTORY} )

# configure_file(client/path_config.h.in ${CMAKE_BINARY_DIR}/generated/path_config.h)
# include_directories(${CMAKE_BINARY_DIR}/generated/)

file(GLOB_RECURSE SRC CONFIGURE_DEPENDS "*.cpp")

# Create the executable

add_executable(server ${SRC})
target_link_libraries(server PRIVATE Qt5::Core protobuf::libprotobuf-lite ${OPENSSL_CRYPTO_LIBRARY} ZLIB::ZLIB woodstock PkgConfig::TCMALLOC ${PROTOCOL_LIBRARY}) 
