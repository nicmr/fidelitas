FROM rust:1-buster

# rust minggw windows target
RUN rustup target add x86_64-pc-windows-gnu

# Update package list
RUN apt-get update

# x86_64 mingw
RUN apt-get -y install gcc-mingw-w64-x86-64

# install 7z
RUn apt-get -y install p7zip

#install libvlc
RUN wget download.videolan.org/pub/videolan/vlc/3.0.8/win64/vlc-3.0.8-win64.7z

RUN p7zip -d vlc-3.0.8-win64.7z


COPY ./ /fidelitas/

WORKDIR /fidelitas

# move vlc.lib to /fidelitas/lib directory
RUN mv /vlc-3.0.8/sdk/lib/libvlc.lib ./lib/libvlc.lib

RUN ln ./lib/libvlc.lib ./lib/vlc.lib

# fix rustup.rs MinGW bug
# reference: https://wiki.archlinux.org/index.php/Rust#Windows
RUN /bin/bash /fidelitas/scripts/fix-minggw.sh

ENV RUSTFLAGS="-C link-args=-L/fidelitas/lib"

CMD cargo build --target=x86_64-pc-windows-gnu \
    && cp /fidelitas/target/x86_64-pc-windows-gnu/debug/fidelitas.exe /artifacts \
    && mv /vlc-3.0.8/libvlc.dll /artifacts \
    && mv /vlc-3.0.8/libvlccore.dll /artifacts \
    && ls /artifacts