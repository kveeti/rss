# 1.28.1-alpine3.23-slim
# arm64
# FROM nginx@sha256:366f9af49d466b994e6f4a69c9c2ee9f8a10a2bea925e2737d510c377c220688
# amd64
FROM nginx@sha256:2c71a793b50ad6d31b5259d2c987769303b0acd243ea16c2e7a88ad5e75f57cc

COPY ./dist /usr/share/nginx/html
COPY ./nginx.conf /etc/nginx/nginx.conf.template

EXPOSE 80

CMD ["/bin/sh", "-c", "envsubst '$API_BASE_URL' < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf && exec nginx -g 'daemon off;'"]
