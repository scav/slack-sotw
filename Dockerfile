FROM rust:1.41.0 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release

FROM rust:slim-buster
COPY --from=build-env /app/target/release/slack-sotw /
RUN apt-get update && apt-get install -y postgresql
USER nobody
CMD ["./slack-sotw"]