# The media container: Node with sharp, ffmpeg, poppler and Chromium, running worker/container/server.mjs.
# Built from the repository root (wrangler.jsonc sets the build context); Dockerfile.dockerignore keeps the
# library's data out of the image.
FROM node:24-bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates ffmpeg poppler-utils chromium \
     fonts-liberation2 fonts-noto-core fonts-noto-cjk fonts-noto-color-emoji \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts && npm cache clean --force
COPY lib ./lib
COPY worker/container/server.mjs ./worker/container/server.mjs
ENV NODE_ENV=production VERIFIER_CHROMIUM=/usr/lib/chromium/chromium
USER node
EXPOSE 8080
CMD ["node", "worker/container/server.mjs"]
