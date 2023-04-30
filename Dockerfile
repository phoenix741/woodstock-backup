FROM node:20 as dependencies
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src/nestjs
COPY nestjs/package.json /src/nestjs
COPY nestjs/package-lock.json /src/nestjs
RUN npm ci --production

#
# -------- Build front -------
FROM dependencies as build-front

WORKDIR /src/front
COPY front/package.json /src/front
COPY front/package-lock.json /src/front
RUN npm ci

COPY front/ /src/front/
RUN npm run build 

#
# -------- Build back -------
FROM dependencies as build-back

WORKDIR /src/nestjs
RUN npm ci

COPY nestjs/ /src/nestjs/
RUN npm run buildall

#
# -------- Dist -----------
FROM node:20 AS dist

WORKDIR /nestjs

RUN npm install pm2 -g

COPY --from=dependencies /src/nestjs/node_modules /nestjs/node_modules
COPY --from=build-back /src/nestjs/config/ /nestjs/config/
COPY --from=build-back /src/nestjs/dist/ /nestjs/
COPY --from=build-back /src/nestjs/woodstock.proto /nestjs/
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