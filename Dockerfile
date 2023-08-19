# ------------ TS Builder
FROM node:20-alpine@sha256:f62abc08fe1004555c4f28b6793af8345a76230b21d2d249976f329079e2fef2 as builder

ENV TZ Europe/Amsterdam

WORKDIR /app

COPY pnpm-lock.yaml pnpm-lock.yaml
COPY package.json package.json

RUN corepack enable && \
    pnpm install

ENV NODE_ENV production

COPY tsconfig.json tsconfig.json
COPY src src

RUN pnpm build
# ------------ END TS Builder

FROM node:20-alpine@sha256:f62abc08fe1004555c4f28b6793af8345a76230b21d2d249976f329079e2fef2 as runner

RUN apk add --no-cache git bash

RUN git config --global user.email bot@interactivestudios.nl && \
    git config --global user.name bot

ENV DATABASE_PATH ${DATABASE_PATH:-database}
ENV NODE_ENV production

WORKDIR /app

COPY package.json package.json

COPY pnpm-lock.yaml pnpm-lock.yaml

ENV NODE_ENV production
ENV TZ Europe/Amsterdam

RUN corepack enable && \
    pnpm install -P

COPY --from=builder /app/build build

# Do not run this through yarn / npm for correct SIG handling
CMD ["node", "build/index.js"]
