FROM alpine as build
RUN apk add -U curl
ENV VERSION 0.81.0
RUN curl -sSL https://github.com/gohugoio/hugo/releases/download/v${VERSION}/hugo_${VERSION}_Linux-64bit.tar.gz -o /tmp/hugo.tar.gz && \
        tar zxf /tmp/hugo.tar.gz -C /usr/local/bin
COPY . /app
WORKDIR /app
RUN /usr/local/bin/hugo --gc

FROM nginx:alpine
COPY --from=build /app/public /usr/share/nginx/html
