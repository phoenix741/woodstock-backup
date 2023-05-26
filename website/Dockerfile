#
# -------- Build --------
FROM node:14-alpine as build
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src
COPY package*.json /src/
RUN npm install

COPY . /src/

RUN npm run build

#
# -------- Dist -----------
FROM nginx:mainline-alpine as production-stage
COPY --from=build /src/.vuepress/dist/ /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
