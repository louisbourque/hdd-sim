FROM ubuntu:24.04

RUN apt update && apt install -y build-essential curl pkg-config \
    libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev \
    libfontconfig-dev libfreetype-dev libxcb1-dev libasound2-dev libdbus-1-dev

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH

WORKDIR /mnt

CMD ["/bin/bash"]
