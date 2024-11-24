#
# -------- Build --------
FROM node:22 AS build
LABEL MAINTAINER="Ulrich Van Den Hekke <ulrich.vdh@shadoware.org>"

WORKDIR /src
COPY package*.json /src/
RUN npm install

COPY . /src/

RUN npm run build

#
# -------- Dist -----------
FROM nginx:stable-alpine AS production-stage
COPY --from=build /src/.vitepress/dist/ /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
