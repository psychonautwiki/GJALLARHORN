FROM psychonaut/rust-nightly:latest

ADD . /my-source

RUN    cd /my-source \
    && cargo build -v --release \
    && mv /my-source/target/release/gjallarhorn /gjallarhorn \
    && rm -rfv /my-source

CMD ["/gjallarhorn"]
