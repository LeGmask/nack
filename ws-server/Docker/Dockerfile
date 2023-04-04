FROM rust:1.67

WORKDIR /usr/src/ws-server
COPY . .

RUN cargo install --path .

CMD ["ws-server"]