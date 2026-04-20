FROM rust:1.88-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        libssl-dev \
        pkg-config \
        procps \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 dev \
    && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash dev

ENV BUN_INSTALL=/usr/local/bun

RUN curl -fsSL https://bun.sh/install | bash \
    && ln -sf /usr/local/bun/bin/bun /usr/local/bin/bun \
    && ln -sf /usr/local/bun/bin/bunx /usr/local/bin/bunx

RUN ln -sf /usr/local/cargo/bin/cargo /usr/local/bin/cargo \
    && ln -sf /usr/local/cargo/bin/rustc /usr/local/bin/rustc \
    && ln -sf /usr/local/cargo/bin/rustup /usr/local/bin/rustup

RUN mkdir -p /home/dev/.cargo /workspace-root \
    && chown -R dev:dev /home/dev /usr/local/cargo /usr/local/bun /workspace-root

ENV HOME=/home/dev
ENV CARGO_HOME=/usr/local/cargo

WORKDIR /workspace-root/underlay-reference

CMD ["sleep", "infinity"]
