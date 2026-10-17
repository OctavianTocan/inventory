# The media container on the Rust binary: `mymind container` with the libvips build sharp ships, ffmpeg, poppler and
# Chromium, and no Node. Built from the repository root (crates/worker/wrangler.jsonc sets the build context);
# Dockerfile.dockerignore lets only the Rust workspace in. Bookworm, as the JS image: the same Chromium and fonts draw
# the same link screenshots.
FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY rust/fetch-libvips.sh ./
RUN ./fetch-libvips.sh linux-x64
COPY rust/Cargo.toml rust/Cargo.lock ./
COPY rust/crates ./crates
RUN cargo build --release --locked -p mymind

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates ffmpeg poppler-utils chromium \
     fonts-liberation2 fonts-noto-core fonts-noto-cjk fonts-noto-color-emoji \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --create-home media
COPY --from=build /src/libvips/linux-x64/libvips-cpp.so.8.17.3 /usr/local/lib/
RUN ldconfig
COPY --from=build /src/target/release/mymind /usr/local/bin/mymind
ENV VERIFIER_CHROMIUM=/usr/lib/chromium/chromium
USER media
EXPOSE 8080
CMD ["mymind", "container"]
