FROM ubuntu:mantic
ARG RUST_VERSION=1.75.0
ENV RUST_VERSION=$RUST_VERSION

RUN apt update
RUN apt install libgtk-4-dev build-essential libadwaita-1-dev curl -y

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN . ~/.cargo/env
RUN ls $HOME/.cargo/env
ENV PATH=/root/.cargo/bin:$PATH
RUN rustup install ${RUST_VERSION}


ENV APPIMAGE_VERSION=13
ENV APPIMAGE_EXTRACT_AND_RUN=1

RUN cargo install cargo-appimage

RUN apt install wget -y

RUN wget https://github.com/AppImage/AppImageKit/releases/download/$APPIMAGE_VERSION/appimagetool-x86_64.AppImage
RUN chmod +x appimagetool-x86_64.AppImage
RUN ./appimagetool-x86_64.AppImage --appimage-extract
RUN ls
RUN ln -nfs /squashfs-root/usr/bin/appimagetool /usr/bin/appimagetool

RUN apt install file desktop-file-utils appstream -y

WORKDIR /mnt

CMD ["/bin/bash"]
