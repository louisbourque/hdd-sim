FROM ubuntu:mantic
ARG RUST_VERSION=1.75.0
ENV RUST_VERSION=$RUST_VERSION

RUN apt update
RUN apt install build-essential curl libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev -y

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
RUN apt install -y nodejs

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH
RUN rustup install ${RUST_VERSION}

WORKDIR /mnt

CMD ["/bin/bash"]
