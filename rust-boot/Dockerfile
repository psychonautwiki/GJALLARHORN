FROM psychonaut/rust-nightly:latest

ADD . /my-source

RUN    cd /my-source \
    && cargo build -v --release \
    && mv /my-source/target/release/boot /eye_of_providence \
    && rm -rfv /my-source

CMD ["/boot"]
