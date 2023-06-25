ARG SRC="ghcr.io/cross-rs/arm-unknown-linux-gnueabihf:0.2.5"
#ARG SRC="rustembedded/cross:arm-unknown-linux-gnueabihf-0.2.1"
FROM $SRC

ENV firmware="1.20230405"

RUN apt-get update \
    && apt-get install -y wget

RUN wget -O - https://github.com/raspberrypi/firmware/archive/refs/tags/$firmware.tar.gz \
    | tar -xzf - -C / --strip-components 2 firmware-$firmware/hardfp/opt/vc

RUN apt-get install -y llvm-dev libclang-dev clang

ENV LLVM_CONFIG_PATH=/usr/bin/llvm-config
