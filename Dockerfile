FROM debian:buster
WORKDIR app
COPY target/x86_64-unknown-linux-gnu/release/sparebank1-to-ynab-sync /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]
