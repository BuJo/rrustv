FROM debian:bookworm as build

RUN apt-get update \
    && mkdir /build

# RISCV-GNU Toolchain
WORKDIR /build
RUN apt-get install -y autoconf automake autotools-dev curl python3 libmpc-dev \
      libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool \
      patchutils bc zlib1g-dev libexpat-dev git \
    && git clone --recursive https://github.com/riscv/riscv-gnu-toolchain.git \
    && git clone --recursive https://github.com/riscv/riscv-opcodes.git

WORKDIR /build/riscv-gnu-toolchain

RUN ./configure --prefix=/opt/riscv --with-arch=rv32gc --with-abi=ilp32d \
    && make -j$(nproc) \
    && /opt/riscv/bin/riscv32-unknown-elf-objdump --version \
    && make clean

RUN ./configure --prefix=/opt/riscv --with-arch=rv64gc --with-abi=lp64d \
    && make -j$(nproc) \
    && /opt/riscv/bin/riscv64-unknown-elf-objdump --version \
    && make clean

# SAIL
WORKDIR /build
RUN apt-get install -y device-tree-compiler

RUN git clone https://github.com/riscv-software-src/riscv-isa-sim.git

WORKDIR /build/riscv-isa-sim
RUN mkdir build && cd build \
    && ../configure --prefix=/opt/riscv \
    && make \
    && make install

FROM python:3-bookworm

# RISCV-GNU Toolchain
# SPIKE RISCV sim
COPY --from=build /opt/riscv /opt/riscv

RUN apt-get update && apt-get install -y device-tree-compiler && apt-get clean

# RISCOF
RUN pip3 install riscof \
    && riscof --version

ENV PATH="/opt/riscv/bin:$PATH"

WORKDIR /work
ENTRYPOINT ["/usr/local/bin/riscof"]
CMD ["run","--no-browser","--config=config.ini","--suite=riscv-arch-test/riscv-test-suite/","--env=riscv-arch-test/riscv-test-suite/env"]
