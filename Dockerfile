FROM node:10 as dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src/server
COPY server/package.json /src/server
COPY server/package-lock.json /src/server
RUN npm install --production

#
# -------- Build --------
FROM dependencies as build

WORKDIR /src/client
COPY client/package.json /src/client
COPY client/package-lock.json /src/client
RUN npm install

WORKDIR /src/server
RUN npm install

COPY . /src/

WORKDIR /src/client

ENV VUE_APP_GRAPHQL_HTTP=/graphql

RUN npm run build -- --prod

WORKDIR /src/server
RUN npm run build 

#
# -------- Dist -----------
FROM node:10 AS dist

RUN apt update && apt install -y btrfs-compsize btrfs-prog && rm -rf /var/lib/apt/lists/*

WORKDIR /server
COPY --from=build /src/server/dist/ /server/
COPY --from=build /src/client/dist /server/client/
COPY --from=dependencies /src/server/node_modules /server/node_modules

ENV STATIC_PATH=/server/client/
ENV NODE_ENV=production
ENV BACKUP_PATH=/backups
ENV LOG_LEVEL=info
ENV REDIS_HOST=redis
ENV REDIS_PORT=6379

ENV VUE_APP_GRAPHQL_HTTP=/graphql

EXPOSE 3000