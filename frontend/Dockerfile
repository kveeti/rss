ARG NGINX_IMAGE_HASH
FROM nginx@${NGINX_IMAGE_HASH}

COPY ./dist /usr/share/nginx/html
COPY ./nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
