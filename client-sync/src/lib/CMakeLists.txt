set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_CXX_STANDARD 20)

# Find package

find_package(ZLIB REQUIRED)
find_package(protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(Threads)
find_package(OpenSSL REQUIRED)
find_package(Qt5 COMPONENTS Core REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(TCMALLOC REQUIRED IMPORTED_TARGET libtcmalloc_minimal)
include(GNUInstallDirs)

include_directories(${Protobuf_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
include_directories(${TCMALLOC_INCLUDE_DIRS})

file(GLOB_RECURSE SRC CONFIGURE_DEPENDS "*.cpp")
set(PROTO ${CMAKE_CURRENT_SOURCE_DIR}/woodstock.proto)

# Configuration GRPC

get_target_property(grpc_cpp_plugin_location gRPC::grpc_cpp_plugin LOCATION)

# Create the library

add_library(woodstock STATIC ${SRC} ${PROTO})
target_link_libraries(woodstock PRIVATE gRPC::grpc++_reflection Qt5::Core protobuf::libprotobuf-lite ZLIB::ZLIB PkgConfig::TCMALLOC) # Link to lite only
protobuf_generate(TARGET woodstock LANGUAGE cpp)
protobuf_generate(TARGET woodstock LANGUAGE grpc GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc PLUGIN "protoc-gen-grpc=${grpc_cpp_plugin_location}")
target_include_directories (woodstock PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

# set_target_properties(woodstock PROPERTIES PUBLIC_HEADER include/mylib.h)
set_target_properties(woodstock PROPERTIES VERSION ${PROJECT_VERSION})
set_target_properties(woodstock PROPERTIES SOVERSION ${PROJECT_VERSION_MAJOR})

# Installation

#install(TARGETS woodstock LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR} PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
