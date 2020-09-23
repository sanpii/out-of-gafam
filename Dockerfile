FROM rust:latest

ENV APP_ENVIRONMENT=prod
ENV BOWER_FLAGS=--allow-root

COPY . .
RUN apt update \
    && apt install -y npm \
    && npm install -g bower
RUN make
CMD ["target/release/oog"]
