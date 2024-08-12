#
# -------- Base Rust -----------
ARG RUST_VERSION=1-alpine
ARG NODE_VERSION=20-alpine
ARG FEATURES=all

FROM rust:$RUST_VERSION AS build-chef
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN if cat /etc/os-release | grep -q 'ID=alpine'; then \
  apk add --no-cache musl-dev clang clang-dev alpine-sdk cmake protoc acl-dev nodejs npm; \
  else \
  apt-get update && apt-get install -y cmake protobuf-c-compiler protobuf-codegen protobuf-compiler libacl1-dev nodejs npm; \
  fi

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
COPY ./shared-rs /src/shared-rs/

RUN cargo build --release $FEATURES
WORKDIR /src/shared-rs
RUN npm install && npm run build

#
# -------- Dependencies -------

FROM node:$NODE_VERSION AS dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src

RUN mkdir -p /src/{nestjs,shared-rs,front,docs} && mkdir -p /src/docs/website
COPY package*.json /src
COPY front/package*.json /src/front/
COPY nestjs/package*.json /src/nestjs/
COPY shared-rs/package*.json /src/shared-rs/
COPY docs/website/package*.json /src/docs/website/

RUN npm ci

FROM dependencies AS prod-dependencies

WORKDIR /src
RUN npm ci --production

#
# -------- Build front -------
FROM dependencies AS build-front

WORKDIR /src/front
COPY front/ /src/front/
RUN npm run build 

#
# -------- Build back -------
FROM dependencies AS build-back

WORKDIR /src/nestjs

COPY --from=build-sharedrs /src/shared-rs/* /src/shared-rs/
COPY nestjs/ /src/nestjs/
RUN npm run buildall

#
# -------- Dist -----------
FROM node:$NODE_VERSION AS dist

RUN if cat /etc/os-release | grep -q 'ID=alpine'; then \
  apk add --no-cache acl; \
  else \
  apt-get update && apt-get install -y libacl1 && apt-get clean && rm -rf /var/lib/apt/lists/*; \
  fi

RUN npm install pm2 -g

WORKDIR /app/nestjs

COPY --from=build-sharedrs /src/target/release/ws_backuppc_importer /app/cli/
COPY --from=build-sharedrs /src/target/release/ws_client_daemon /app/cli/
COPY --from=build-sharedrs /src/target/release/ws_console /app/cli/
COPY --from=build-sharedrs /src/target/release/ws_sync /app/cli/

COPY --from=build-sharedrs /src/shared-rs/index.* /app/shared-rs/
COPY --from=build-sharedrs /src/shared-rs/shared-rs.* /app/shared-rs/

COPY --from=prod-dependencies /src/nestjs/node_modules /app/nestjs/node_modules
COPY --from=prod-dependencies /src/node_modules /app/node_modules
COPY --from=build-back /src/nestjs/package*.json /app/nestjs/
COPY --from=build-back /src/nestjs/config/ /app/nestjs/config/
COPY --from=build-back /src/nestjs/dist/ /app/nestjs/
COPY --from=build-back /src/nestjs/ecosystem.config.js /app/nestjs/
COPY --from=build-front /src/front/dist /app/nestjs/front/

ENV STATIC_PATH=/app/nestjs/front/
ENV NODE_ENV=production
ENV BACKUP_PATH=/backups
ENV LOG_LEVEL=info
ENV REDIS_HOST=redis
ENV REDIS_PORT=6379

ENV VUE_APP_GRAPHQL_HTTP=/graphql

VOLUME [ "/backups" ]
ENTRYPOINT [ "pm2-runtime" ]
CMD [ "ecosystem.config.js" ]
EXPOSE 3000