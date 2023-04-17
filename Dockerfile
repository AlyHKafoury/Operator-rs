FROM cgr.dev/chainguard/static
WORKDIR /app
COPY --chown=nonroot:nonroot ./target/x86_64-unknown-linux-musl/release/operator-rs /app/
EXPOSE 8080
ENTRYPOINT ["/app/operator-rs"]