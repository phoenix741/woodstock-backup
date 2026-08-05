#
# -------- Base Rust -----------
ARG RUST_VERSION=1
ARG NODE_VERSION=24-slim
ARG DEBIAN_VERSION=trixie
ARG CARGO_ARGS="--workspace --bins"

FROM rust:$RUST_VERSION-$DEBIAN_VERSION AS build-chef
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt-get update && apt-get install -y --no-install-recommends \
  clang \
  libclang-dev \
  build-essential \
  cmake \
  protobuf-compiler \
  libacl1-dev \
  libssl-dev \
  pkg-config \
  curl \
  && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked && rm -rf $CARGO_HOME/registry/

FROM build-chef AS planner

WORKDIR /src/
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#
# -------- Build shared-rs -----------

FROM build-chef AS build-sharedrs

WORKDIR /src/
COPY --from=planner /src/recipe.json /src/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.* /src/
COPY ./woodstock-rs /src/woodstock-rs/
COPY ./client-rs /src/client-rs/
COPY ./cli-rs /src/cli-rs/
COPY ./backuppc-importer-rs /src/backuppc-importer-rs/
COPY ./server-rs /src/server-rs/
COPY ./e2e-tests /src/e2e-tests/

RUN cargo build --release $CARGO_ARGS

# Strip binaries to reduce image size
RUN strip /src/target/release/api_server \
  /src/target/release/client_api_server \
  /src/target/release/job_worker \
  /src/target/release/scheduler \
  /src/target/release/ws_client_daemon \
  /src/target/release/ws_client_console \
  /src/target/release/ws_backuppc_importer \
  /src/target/release/ws_console \
  /src/target/release/ws_sync \
  /src/target/release/ws_restore

#
# -------- Dependencies -------

FROM node:$NODE_VERSION AS dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src

RUN mkdir -p /src/{front,docs} && mkdir -p /src/docs/website
COPY package*.json /src
COPY front/package*.json /src/front/
COPY docs/website/package*.json /src/docs/website/

RUN npm ci

#
# -------- Build front -------
FROM dependencies AS build-front

WORKDIR /src/front
COPY front/ /src/front/
RUN npm run build 

#
# -------- Build client -------
FROM debian:$DEBIAN_VERSION-slim AS client

RUN apt-get update && apt-get install -y --no-install-recommends \
  acl \
  btrfs-progs \
  ca-certificates \
  fuse3 \
  libacl1 \
  libfuse2 \
  liblzma5 \
  libssl3 \
  samba-common-bin \
  smbclient \
  tzdata \
  && rm -rf /var/lib/apt/lists/*

# Create a user to run the app
ARG APP_USER=woodstock
ARG APP_UID=1000
RUN groupadd --gid $APP_UID $APP_USER && \
  useradd --uid $APP_UID --gid $APP_UID --create-home --shell /usr/sbin/nologin $APP_USER && \
  mkdir -p /app/cli /etc/woodstock && \
  chown -R $APP_USER:$APP_USER /app /etc/woodstock

# Ensure the client looks for config in the volume mount point
ENV CLIENT_PATH=/etc/woodstock

COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/ws_client_daemon /app/cli/

VOLUME [ "/etc/woodstock" ]

USER $APP_USER
CMD [ "/app/cli/ws_client_daemon" ]

#
# -------- Server (Rust) -----------
FROM debian:$DEBIAN_VERSION-slim AS server

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  libacl1 \
  libfuse2 \
  liblzma5 \
  libssl3 \
  tzdata \
  && rm -rf /var/lib/apt/lists/*

# Create a user to run the app
ARG APP_USER=woodstock
ARG APP_UID=1000
RUN groupadd --gid $APP_UID $APP_USER && \
  useradd --uid $APP_UID --gid $APP_UID --create-home --shell /usr/sbin/nologin $APP_USER && \
  mkdir -p /app /backups && \
  chown -R $APP_USER:$APP_USER /app /backups

WORKDIR /app

# Copy Rust binaries
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/api_server /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/client_api_server /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/job_worker /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/scheduler /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/ws_backuppc_importer /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/ws_console /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/ws_sync /app/
COPY --chown=$APP_USER:$APP_USER --from=build-sharedrs /src/target/release/ws_restore /app/

# Copy Frontend static files
COPY --chown=$APP_USER:$APP_USER --from=build-front /src/front/dist /app/static

ENV STATIC_PATH=/app/static
ENV BACKUP_PATH=/backups
ENV LOG_LEVEL=info
ENV REDIS_HOST=redis
ENV REDIS_PORT=6379

VOLUME [ "/backups" ]

USER $APP_USER

# Default to API server
CMD [ "/app/api_server" ]
EXPOSE 3000 8443
