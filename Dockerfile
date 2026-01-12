FROM docker.io/devkitpro/devkitarm:latest

# env
ENV MAKEFLAGS="-j$(nproc)"
ENV PATH="$DEVKITPRO/tools/bin:$PATH"
ENV PATH="$DEVKITARM/bin:$PATH"
ENV PATH="/root/.cargo/bin:$PATH"
ENV DEVKITPRO=/opt/devkitpro
ENV DEVKITARM=/opt/devkitpro/devkitARM
ENV DEVKITPPC=/opt/devkitpro/devkitPPC

# rust
# this part takes a while for some reason idk why
# if it looks like its stuck just wait it will finish
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    bash -s -- \
    --component rust-src \
    --default-toolchain nightly \
    --target armv7a-none-eabi -y
RUN cargo install cargo-3ds
# if you dont need rust remove everything between the "rust" comment and this comment

# deps
RUN apt-get -y update && \
    apt-get -y install g++ \
    gcc \
    git \
    make \
    zip \
    wget \
    clang

# build bannertool for banner creation
RUN git clone https://github.com/Epicpkmn11/bannertool /tmp/bannertool
RUN cd /tmp/bannertool && \
    git submodule init && \
    git config submodule.buildtools.url https://github.com/Steveice10/buildtools && \
    git -c protocol.file.allow=always submodule update && \
    make && \
    sudo make install && \
    rm -rf /tmp/bannertool

# download makerom
RUN cd /tmp && \
    wget https://github.com/3DSGuy/Project_CTR/releases/download/makerom-v0.18.3/makerom-v0.18.3-ubuntu_x86_64.zip && \
    unzip makerom-v0.18.3-ubuntu_x86_64.zip && \
    sudo install makerom /usr/local/bin/ && \
    rm makerom-v0.18.3-ubuntu_x86_64.zip makerom
