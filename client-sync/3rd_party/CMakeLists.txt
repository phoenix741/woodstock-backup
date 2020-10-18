cmake_minimum_required(VERSION 3.13.0)
cmake_policy(SET CMP0054 NEW)

project(3rd_party)
include(ExternalProject)

message("-- Call QtService")
ExternalProject_Add(QtService
    GIT_REPOSITORY "https://github.com/phoenix741/QtService.git"
    GIT_TAG "2.0.2-100"
    CONFIGURE_COMMAND qmake
    BUILD_COMMAND make qmake_all COMMAND make sub-src-all
    INSTALL_COMMAND make sub-src-install_subtargets INSTALL_ROOT="${CMAKE_CURRENT_BINARY_DIR}/../vcpkg_installed/${VCPKG_TARGET_TRIPLET}"
    BUILD_IN_SOURCE 1
)

if(${WITH_PROTOCOL_HTTP2})
message("-- Call NgHttp2")
ExternalProject_Add(nghttp2
                    GIT_REPOSITORY "https://github.com/nghttp2/nghttp2.git"
                    GIT_TAG "v1.42.0"
                    INSTALL_DIR ${CMAKE_CURRENT_BINARY_DIR}/../vcpkg_installed/${VCPKG_TARGET_TRIPLET}
                    CONFIGURE_COMMAND ./configure --prefix=<INSTALL_DIR> --with-boost=${CMAKE_CURRENT_BINARY_DIR}/../vcpkg_installed/${VCPKG_TARGET_TRIPLET} --enable-asio-lib --enable-static --disable-shared
                    BUILD_COMMAND make
                    BUILD_IN_SOURCE 1)

ExternalProject_Add_Step(nghttp2
        bootstrap
        COMMAND autoreconf -i && automake && autoconf
        DEPENDEES download
        DEPENDERS configure
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/nghttp2-prefix/src/nghttp2)
endif()