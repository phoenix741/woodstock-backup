FROM rust:1 AS build-chef

# Install musl-dev on Alpine to avoid error "ld: cannot find crti.o: No such file or directory"
RUN ((cat /etc/os-release | grep ID | grep alpine) && apk add --no-cache musl-dev || true) \
  && cargo install cargo-chef 

FROM build-chef AS planner

WORKDIR /src/
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM build-chef AS build-sharedrs

RUN apt-get update && apt-get install -y cmake protobuf-c-compiler protobuf-codegen protobuf-compiler libacl1-dev nodejs npm && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /src/
COPY --from=planner /src/recipe.json /src/recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.* /src/
COPY ./clientrs /src/clientrs/
COPY ./backuppc_importer /src/backuppc_importer/
COPY ./shared-rs /src/shared-rs/

RUN cargo build --release --no-default-features -F pool,client,server,acl,xattr
WORKDIR /src/shared-rs
RUN npm install && npm run build

FROM node:20 AS dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src/nestjs
COPY nestjs/package.json /src/nestjs
COPY nestjs/package-lock.json /src/nestjs
RUN npm ci --production

#
# -------- Build front -------
FROM dependencies AS build-front

WORKDIR /src/front
COPY front/package.json /src/front
COPY front/package-lock.json /src/front
RUN npm ci

COPY front/ /src/front/
RUN npm run build 

#
# -------- Build back -------
FROM dependencies AS build-back

WORKDIR /src/nestjs
RUN npm ci

COPY --from=build-sharedrs /src/shared-rs/* /src/shared-rs/
COPY nestjs/ /src/nestjs/
RUN npm run buildall

#
# -------- Dist -----------
FROM node:20 AS dist

WORKDIR /nestjs

RUN npm install pm2 -g

COPY --from=build-sharedrs /src/shared-rs/* /shared-rs/
COPY --from=dependencies /src/nestjs/node_modules /nestjs/node_modules
COPY --from=build-back /src/nestjs/config/ /nestjs/config/
COPY --from=build-back /src/nestjs/dist/ /nestjs/
COPY --from=build-back /src/nestjs/ecosystem.config.js /nestjs/
COPY --from=build-front /src/front/dist /nestjs/front/

ENV STATIC_PATH=/nestjs/front/
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