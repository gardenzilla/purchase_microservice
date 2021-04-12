FROM fedora:33
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/purchase_microservice /usr/local/bin/purchase_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["purchase_microservice"]
