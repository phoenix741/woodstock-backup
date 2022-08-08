FROM node:16 as dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src/nestjs
COPY nestjs/package.json /src/nestjs
COPY nestjs/package-lock.json /src/nestjs
RUN npm install --production

#
# -------- Build --------
FROM dependencies as build

WORKDIR /src/front
COPY front/package.json /src/front
COPY front/package-lock.json /src/front
RUN npm install

WORKDIR /src/nestjs
RUN npm install

COPY front/ /src/front/
COPY nestjs/ /src/nestjs/

WORKDIR /src/front

ENV VUE_APP_GRAPHQL_HTTP=/graphql

RUN npm run build -- --prod

WORKDIR /src/nestjs
RUN npm run build:all

#
# -------- Dist -----------
FROM node:16 AS dist

WORKDIR /nestjs

COPY --from=dependencies /src/nestjs/node_modules /nestjs/node_modules
COPY --from=build /src/nestjs/config/ /nestjs/config/
COPY --from=build /src/nestjs/dist/ /nestjs/
COPY --from=build /src/nestjs/woodstock.proto /nestjs/
COPY --from=build /src/front/dist /nestjs/front/

ENV STATIC_PATH=/nestjs/front/
ENV NODE_ENV=production
ENV BACKUP_PATH=/backups
ENV LOG_LEVEL=info
ENV REDIS_HOST=redis
ENV REDIS_PORT=6379

ENV VUE_APP_GRAPHQL_HTTP=/graphql

ENTRYPOINT [ "node" ]
CMD [ "/nestjs/apps/api/main.js" ]
EXPOSE 3000