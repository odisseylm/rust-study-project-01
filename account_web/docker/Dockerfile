FROM rust:1.79.0-slim-bullseye AS build
ARG RELEASE_OR_DEBUG_BUILD_PARAM=--release
ARG RELEASE_OR_DEBUG_TARGET_DIR=./target/release
ARG RUN_RUST_BACKTRACE=full

EXPOSE 80 8080 443 8443 8000 2735 3000

LABEL rust-test-label-02=value-2

RUN apt-get -qq update &&                       \
    apt-get -qq install pkg-config &&           \
    apt-get -qq install openssl &&              \
    apt-get -qq update && apt-get -qq install libssl-dev

# View app name in Cargo.toml
#ARG APP_NAME=mvv_temp

#WORKDIR /app
WORKDIR /build

RUN echo "Build mode '${RELEASE_OR_DEBUG_BUILD_PARAM:-debug}'"

# Optimizations
#  * Reuse cache layer for build faster

#COPY . .

# COPY Cargo.lock Cargo.toml build.rs ./
COPY Cargo.lock Cargo.toml ./

RUN    mkdir ./src                                                 \
    && mkdir ./account_soa                                         \
    && mkdir ./account_soa/src                                     \
    && mkdir ./account_soa/src/bin                                 \
    && mkdir ./account_web                                         \
    && mkdir ./account_web/src                                     \
    && mkdir ./account_web/src/bin                                 \
    && mkdir ./auth                                                \
    && mkdir ./auth/src                                            \
    && mkdir ./common                                              \
    && mkdir ./common/src                                          \
    && mkdir ./proc_macro                                          \
    && mkdir ./proc_macro/src                                      \
    && mkdir ./error_macro                                         \
    && mkdir ./error_macro/src                                     \
    && mkdir ./tuple_heter_iter                                    \
    && mkdir ./tuple_heter_iter/src                                \
    && mkdir ./tuple_heter_iter_macro                              \
    && mkdir ./tuple_heter_iter_macro/src                          \
    && mkdir ./xtask                                               \
    && mkdir ./xtask/src                                           \
    && echo "fn main() { }" > ./account_soa/src/bin/main.rs        \
    && echo "// dummy file" > ./account_soa/src/lib.rs             \
    && echo "fn main() { }" > ./account_web/src/bin/main.rs        \
    && echo "// dummy file" > ./account_web/src/lib.rs             \
    && echo "// dummy file" > ./auth/src/lib.rs                    \
    && echo "// dummy file" > ./common/src/lib.rs                  \
    && echo "// dummy file" > ./proc_macro/src/lib.rs              \
    && echo "// dummy file" > ./error_macro/src/lib.rs             \
    && echo "// dummy file" > ./tuple_heter_iter/src/lib.rs        \
    && echo "// dummy file" > ./tuple_heter_iter_macro/src/lib.rs  \
    && echo "fn main() { }" > ./xtask/src/main.rs

COPY target/externals/cargo-patch-subdep-ver  /usr/local/cargo/bin/cargo-patch-subdep-ver/

# COPY error_macro/Cargo.lock error_macro/Cargo.toml ./error_macro/
COPY account_soa/Cargo.toml             ./account_soa/
COPY account_web/Cargo.toml             ./account_web/
COPY auth/Cargo.toml                    ./auth/
COPY common/Cargo.toml                  ./common/
COPY proc_macro/Cargo.toml              ./proc_macro/
COPY error_macro/Cargo.toml             ./error_macro/
COPY tuple_heter_iter/Cargo.toml        ./tuple_heter_iter/
COPY tuple_heter_iter_macro/Cargo.toml  ./tuple_heter_iter_macro/
COPY xtask/Cargo.toml                   ./xtask/


# Build the dependencies without the actual source code to cache dependencies separately
# RUN cargo patch-subdep-ver
RUN /usr/local/cargo/bin/cargo-patch-subdep-ver/cargo-patch-subdep-ver

RUN    cargo check $RELEASE_OR_DEBUG_BUILD_PARAM --workspace --all-targets --all-features \
    && cargo build $RELEASE_OR_DEBUG_BUILD_PARAM --workspace

RUN    rm ./account_soa/src/*.rs                  \
    && rm ./account_soa/src/bin/*.rs              \
    && rm ./account_web/src/*.rs                  \
    && rm ./account_web/src/bin/*.rs              \
    && rm ./auth/src/*.rs                         \
    && rm ./common/src/*.rs                       \
    && rm ./proc_macro/src/*.rs                   \
    && rm ./error_macro/src/*.rs                  \
    && rm ./tuple_heter_iter/src/*.rs             \
    && rm ./tuple_heter_iter_macro/src/*.rs       \
    && rm ./xtask/src/*.rs

#RUN ls --recursive ./src/
COPY account_soa/src/             ./account_soa/src/
COPY account_soa/templates/       ./account_soa/templates/
COPY account_web/src/             ./account_web/src/
COPY account_web/templates/       ./account_web/templates/
COPY auth/src/                    ./auth/src/
COPY auth/templates/              ./auth/templates/
COPY common/src/                  ./common/src/
COPY proc_macro/src/              ./proc_macro/src/
COPY error_macro/src/             ./error_macro/src/
COPY tuple_heter_iter/src/        ./tuple_heter_iter/src/
COPY tuple_heter_iter_macro/src/  ./tuple_heter_iter_macro/src/
COPY xtask/src/                   ./xtask/src/

RUN    touch ./account_soa/src/lib.rs             \
    && touch ./account_web/src/lib.rs             \
    && touch ./auth/src/lib.rs                    \
    && touch ./common/src/lib.rs                  \
    && touch ./proc_macro/src/lib.rs              \
    && touch ./error_macro/src/lib.rs             \
    && touch ./tuple_heter_iter/src/lib.rs        \
    && touch ./tuple_heter_iter_macro/src/lib.rs  \
    && touch ./xtask/src/main.rs


RUN cargo build $RELEASE_OR_DEBUG_BUILD_PARAM --workspace
RUN cp $RELEASE_OR_DEBUG_TARGET_DIR/mvv_account_web /bin/mvv_account_web

##ENV ROCKET_ADDRESS=0.0.0.0
#CMD ["/bin/mvv_account_web"]
#ENTRYPOINT /bin/mvv_account_web arg1 arg2

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bullseye-slim AS final
#FROM debian:buster-slim
#FROM debian:bookworm-slim

ARG RUN_RUST_BACKTRACE=full


RUN adduser                   \
    --disabled-password       \
    --gecos ""                \
    --home "/nonexistent"     \
    --shell "/sbin/nologin"   \
    --no-create-home          \
    --uid "10001"             \
    appuser
USER appuser

COPY --from=build /bin/mvv_account_web /bin/
#ENV ROCKET_ADDRESS=0.0.0.0
#CMD ["/bin/server"]

ENV RUST_BACKTRACE=$RUN_RUST_BACKTRACE
ENTRYPOINT ["/bin/mvv_account_web", "arg1", "arg2"]

# interval=1m
#HEALTHCHECK --interval=5s --timeout=3s CMD wget --quiet --no-check-certificate --tries=1 --spider https://localhost/${PROJECT_CONTEXT_PATH}/api/ping || exit 1
