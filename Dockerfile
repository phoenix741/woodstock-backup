FROM node:12-buster as dependencies
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

COPY client/ /src/client/
COPY server/ /src/server/

WORKDIR /src/client

ENV VUE_APP_GRAPHQL_HTTP=/graphql

RUN npm run build -- --prod

WORKDIR /src/server
RUN npm run build 

#
# -------- Dist -----------
FROM node:12-buster AS dist

RUN apt update && apt install -y btrfs-compsize btrfs-progs coreutils samba-common-bin rsync && rm -rf /var/lib/apt/lists/*

WORKDIR /server
COPY --from=build /src/server/dist/ /server/
COPY --from=build /src/server/config/ /server/config/
COPY --from=build /src/client/dist /server/client/
COPY --from=dependencies /src/server/node_modules /server/node_modules

RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh 
RUN echo "IdentityFile /backups/.ssh/id_rsa" >> /root/.ssh/config
RUN echo "StrictHostKeyChecking=no" >> /root/.ssh/config
RUN mkdir -p /backups/.ssh && chmod 700 /backups/.ssh

ENV STATIC_PATH=/server/client/
ENV NODE_ENV=production
ENV BACKUP_PATH=/backups
ENV LOG_LEVEL=info
ENV REDIS_HOST=redis
ENV REDIS_PORT=6379

ENV VUE_APP_GRAPHQL_HTTP=/graphql

ENTRYPOINT [ "node" ]
CMD [ "/server/main.js" ]
EXPOSE 3000