FROM rust:1.46-buster as builder

RUN USER=root cargo new --lib bragi-status
WORKDIR ./bragi-status
COPY ./Cargo.toml ./Cargo.toml
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && apt-get install -y libjq1 libjq-dev libonig5 libonig-dev \
    && rm -rf /var/lib/apt/lists/*
ENV JQ_LIB_DIR=/usr/lib/x86_64-linux-gnu

RUN cargo build --lib --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/bragi_status*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y netcat \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser \
    SETTINGS=development

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p /opt/bragi-status \
    && mkdir -p /etc/opt/bragi-status

COPY --from=builder /bragi-status/target/release/service /opt/bragi-status/service
COPY --from=builder /bragi-status/config /etc/opt/bragi-status/config
COPY --from=builder /bragi-status/docker/entrypoint.sh /opt/bragi-status/entrypoint.sh

RUN chown -R $APP_USER:$APP_USER /opt/bragi-status

USER $APP_USER
WORKDIR /opt/bragi-status

CMD ["./entrypoint.sh"]
