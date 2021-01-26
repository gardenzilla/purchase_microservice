FROM debian:buster-slim
WORKDIR /usr/local/bin
COPY ./target/release/purchase_microservice /usr/local/bin/purchase_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["purchase_microservice"]