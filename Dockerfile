FROM oven/bun:1.0

ENV TZ Europe/Amsterdam

WORKDIR /app

COPY bun.lockb bun.lockb
COPY package.json package.json

RUN bun install

ENV NODE_ENV production

COPY tsconfig.json tsconfig.json
COPY src src

CMD [ "bun", "start" ]
