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

COPY Cargo.lock Cargo.toml build.rs ./

RUN    mkdir ./src                                                              \
    && mkdir ./src/bin                                                          \
    && mkdir ./_subpackages                                                     \
    && mkdir ./_subpackages/mvv_auth                                            \
    && mkdir ./_subpackages/static_error_macro                                  \
    && mkdir ./_subpackages/tuple_heter_iter                                    \
    && mkdir ./_subpackages/tuple_heter_iter_macro                              \
    && mkdir ./_subpackages/mvv_auth/src                                        \
    && mkdir ./_subpackages/static_error_macro/src                              \
    && mkdir ./_subpackages/tuple_heter_iter/src                                \
    && mkdir ./_subpackages/tuple_heter_iter_macro/src                          \
    && echo "fn main() { }" > ./src/bin/main.rs                                 \
    && echo "// dummy file" > ./src/lib.rs                                      \
    && echo "// dummy file" > ./_subpackages/mvv_auth/src/lib.rs                \
    && echo "// dummy file" > ./_subpackages/static_error_macro/src/lib.rs      \
    && echo "// dummy file" > ./_subpackages/tuple_heter_iter/src/lib.rs        \
    && echo "// dummy file" > ./_subpackages/tuple_heter_iter_macro/src/lib.rs

# COPY static_error_macro/Cargo.lock static_error_macro/Cargo.toml ./static_error_macro/
COPY _subpackages/mvv_auth/Cargo.toml                ./_subpackages/mvv_auth/
COPY _subpackages/static_error_macro/Cargo.toml      ./_subpackages/static_error_macro/
COPY _subpackages/tuple_heter_iter/Cargo.toml        ./_subpackages/tuple_heter_iter/
COPY _subpackages/tuple_heter_iter_macro/Cargo.toml  ./_subpackages/tuple_heter_iter_macro/


# Build the dependencies without the actual source code to cache dependencies separately
RUN    cargo check $RELEASE_OR_DEBUG_BUILD_PARAM --workspace --all-targets --all-features \
    && cargo build $RELEASE_OR_DEBUG_BUILD_PARAM --workspace

RUN rm ./src/*.rs && rm ./src/bin/*.rs                         \
    && rm ./_subpackages/mvv_auth/src/*.rs                     \
    && rm ./_subpackages/static_error_macro/src/*.rs           \
    && rm ./_subpackages/tuple_heter_iter/src/*.rs             \
    && rm ./_subpackages/tuple_heter_iter_macro/src/*.rs

COPY src/ ./src/
COPY templates/ ./templates/
#RUN ls --recursive ./src/
COPY _subpackages/mvv_auth/src/                ./_subpackages/mvv_auth/src/
COPY _subpackages/mvv_auth/templates/          ./_subpackages/mvv_auth/templates/
COPY _subpackages/static_error_macro/src/      ./_subpackages/static_error_macro/src/
COPY _subpackages/tuple_heter_iter/src/        ./_subpackages/tuple_heter_iter/src/
COPY _subpackages/tuple_heter_iter_macro/src/  ./_subpackages/tuple_heter_iter_macro/src/

RUN    touch ./src/lib.rs                                      \
    && touch ./_subpackages/mvv_auth/src/lib.rs                \
    && touch ./_subpackages/static_error_macro/src/lib.rs      \
    && touch ./_subpackages/tuple_heter_iter/src/lib.rs        \
    && touch ./_subpackages/tuple_heter_iter_macro/src/lib.rs


RUN cargo build $RELEASE_OR_DEBUG_BUILD_PARAM --workspace
RUN cp $RELEASE_OR_DEBUG_TARGET_DIR/web_main_1 /bin/web_main_1

##ENV ROCKET_ADDRESS=0.0.0.0
#CMD ["/bin/web_main_1"]
#ENTRYPOINT /bin/web_main_1 arg1 arg2

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bullseye-slim AS final
#FROM debian:buster-slim
#FROM debian:bookworm-slim


RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser
USER appuser

COPY --from=build /bin/web_main_1 /bin/
#ENV ROCKET_ADDRESS=0.0.0.0
#CMD ["/bin/server"]

ENV RUST_BACKTRACE=$RUN_RUST_BACKTRACE
ENTRYPOINT ["/bin/web_main_1", "arg1", "arg2"]

# interval=1m
#HEALTHCHECK --interval=5s --timeout=3s CMD wget --quiet --no-check-certificate --tries=1 --spider https://localhost/${PROJECT_CONTEXT_PATH}/api/ping || exit 1

