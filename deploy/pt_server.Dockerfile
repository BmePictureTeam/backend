FROM debian:10-slim  

RUN apt-get update && apt-get install -y openssl

WORKDIR /app
COPY target/release/pt_server .
RUN chmod +x pt_server

ENTRYPOINT [ "./pt_server" ]