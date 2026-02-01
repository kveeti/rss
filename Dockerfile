FROM scratch
COPY ./backend/backend /usr/local/bin/backend
COPY ./frontend/dist /app/frontend
ENV FRONTEND_DIR=/app/frontend
EXPOSE 8000
ENTRYPOINT ["backend"]
