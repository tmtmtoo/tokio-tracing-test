FROM rust:1.59 as build

# create a new empty shell project
RUN USER=root cargo new --bin tokio_tracing_test
WORKDIR /tokio_tracing_test

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/tokio_tracing_test*
RUN cargo build --release

# our final base
FROM rust:1.59-slim-buster

# copy the build artifact from the build stage
COPY --from=build /tokio_tracing_test/target/release/tokio-tracing-test .

# set the startup command to run your binary
CMD ["./tokio-tracing-test"]
