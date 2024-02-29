FROM debian:buster
RUN apt-get update && apt-get install -y ca-certificates
WORKDIR app
RUN mkdir -p /app/config
RUN mkdir -p /data
COPY target/x86_64-unknown-linux-gnu/release/sparebank1-to-ynab-sync /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]
