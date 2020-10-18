# syntax=docker/dockerfile:experimental

FROM debian:10 as build
LABEL description="Build Docker Woodstock C++"

ENV PIP_CACHE_DIR=/var/cache/buildkit/pip
ENV VCPKG_BINARY_SOURCES=clear;files,/var/cache/vcpkg,readwrite
ENV VCPKG_FEATURE_FLAGS=binarycaching,manifests
ENV CXX=clang++
ENV CC=clang 

RUN mkdir -p ${PIP_CACHE_DIR}
RUN mkdir -p /var/cache/vcpkg

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update \
    && apt-get install -y cmake build-essential git curl zip unzip tar pkg-config fontconfig clang python3-pip gperf libssl-dev libx11-xcb-dev libxkbcommon-x11-dev libsystemd-dev qt5-default qtbase5-dev qtbase5-private-dev qtdeclarative5-dev qtdeclarative5-private-dev autoconf automake libtool

RUN pip3 install qdep \
    && qdep prfgen --qmake /usr/bin/qmake \
    && git config --global user.email "ulrich.vdh@gmail.com" \
    && git config --global user.name "Ulrich Vandenhekke"

WORKDIR /src

RUN --mount=type=cache,target=/var/cache/vcpkg \
    (test -e /src/vcpkg/vcpkg || git clone https://github.com/Microsoft/vcpkg.git) \ 
    && cd vcpkg \
    && ./bootstrap-vcpkg.sh

COPY ./ /src/
WORKDIR /build

RUN cmake -DCMAKE_BUILD_TYPE=Release -DWITH_PROTOCOL_HTTP2=OFF -DWITH_PROTOCOL_GRPC=ON /src \
    && make -j 8

FROM debian:10 as runtime
LABEL description="Build Docker Woodstock C++"

RUN apt-get update && apt-get install fontconfig libqt5core5a libqt5network5 libsystemd0 -y

RUN mkdir /build

COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/lib/libQt5Service.so /usr/lib/x86_64-linux-gnu/libQt5Service.so
COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/lib/libQt5Service.so.2 /usr/lib/x86_64-linux-gnu/libQt5Service.so.2
COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/lib/libQt5Service.so.2.0 /usr/lib/x86_64-linux-gnu/libQt5Service.so.2.0
COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/lib/libQt5Service.so.2.0.2 /usr/lib/x86_64-linux-gnu/libQt5Service.so.2.0.2
COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/plugins/servicebackends/libqstandard.so /usr/lib/x86_64-linux-gnu/qt5/plugins/servicebackends/libqstandard.so
COPY --from=build /build/3rd_party/QtService-prefix/src/QtService/plugins/servicebackends/libqsystemd.so /usr/lib/x86_64-linux-gnu/qt5/plugins/servicebackends/libqsystemd.so

COPY --from=build /build/src/client_daemon/client /build/client
COPY --from=build /build/src/server/server /build/server
COPY --from=build /build/src/tools/readindex/readindex /build/readindex

COPY ./certs /certs/

WORKDIR /build
CMD ./server

VOLUME ["/var/lib/woodstock"]
EXPOSE 3657