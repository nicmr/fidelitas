FROM rust:1-buster

# elm installation
RUN wget https://github.com/elm/compiler/releases/download/0.19.0/binaries-for-linux.tar.gz
RUN tar -xzvf binaries-for-linux.tar.gz
RUN mv elm /usr/bin/elm

# rust armv7 target
RUN rustup target add armv7-unknown-linux-gnueabihf

# Add arm architecture for downloading arm packages
RUN dpkg --add-architecture armhf

# Update package list
# Has to be run  after `dpkg --add-architecture armhf` and before all `apt-get installs` !
RUN apt-get update

# gcc armhf cross compiler
RUN apt-get install -y gcc-arm-linux-gnueabihf

RUN apt-get install -y libvlc-dev:armhf


COPY ./ /fidelitas/

WORKDIR /fidelitas

CMD cargo build --target=armv7-unknown-linux-gnueabihf && cp /fidelitas/target/armv7-unknown-linux-gnueabihf/debug/fidelitas /artifacts && ls /artifacts