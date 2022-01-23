set(CMAKE_TOOLCHAIN_FILE ${CMAKE_CURRENT_SOURCE_DIR}/vcpkg/scripts/buildsystems/vcpkg.cmake CACHE STRING "Vcpkg toolchain file")

option(WITH_PROTOCOL_HTTP2 "Option HTTP/2 REST" OFF)
option(WITH_PROTOCOL_GRPC "Option gRPC" ON)

cmake_minimum_required(VERSION 3.13.0)
cmake_policy(SET CMP0054 NEW)

project(woodstock-client-sync VERSION 0.1 LANGUAGES CXX)
include(CTest)
enable_testing(true)

set(PROTOCOL_LIBRARY)
if(${WITH_PROTOCOL_HTTP2})
  add_definitions(-DWITH_PROTOCOL_HTTP2)
  list(APPEND PROTOCOL_LIBRARY PkgConfig::NGHTTP2_ASIO PkgConfig::NGHTTP2 Boost::system ssl crypto)
  set(WITH_PROTOCOL_GRPC OFF)
endif()

if(${WITH_PROTOCOL_GRPC})
  add_definitions(-DWITH_PROTOCOL_GRPC)
  list(APPEND PROTOCOL_LIBRARY gRPC::grpc++_reflection)
endif()

if (NOT EXISTS "${CMAKE_BINARY_DIR}/3rd_party")
message("execute_process")
file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/3rd_party)
execute_process(
    COMMAND ${CMAKE_COMMAND} -DVCPKG_TARGET_TRIPLET=${VCPKG_TARGET_TRIPLET} -DWITH_PROTOCOL_HTTP2=${WITH_PROTOCOL_HTTP2} -DWITH_PROTOCOL_GRPC=${WITH_PROTOCOL_GRPC} ${CMAKE_SOURCE_DIR}/3rd_party
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/3rd_party
)
execute_process(
    COMMAND ${CMAKE_COMMAND} --build .
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/3rd_party
)
message("end execute_process")
endif()

if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "MSVC")
add_compile_options("/wd4018")
add_compile_options("/wd4146")
add_compile_options("/wd4244")
add_compile_options("/wd4251")
add_compile_options("/wd4267")
add_compile_options("/wd4305")
add_compile_options("/wd4355")
add_compile_options("/wd4800")
add_compile_options("/wd4996")
endif()

add_subdirectory(src)
add_subdirectory(tests)