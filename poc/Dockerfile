FROM node:10.11.0-slim

ADD . /source

WORKDIR /source

RUN npm i -g typescript
RUN npm i; tsc

CMD node dist/app.js
