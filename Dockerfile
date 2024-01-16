ARG BASE_IMAGE_BUILD="rust:buster"
ARG BASE_IMAGE_RUN="gcr.io/distroless/cc"

# Useful guide for cached building:
# https://windsoilder.github.io/writing_dockerfile_in_rust_project.html

FROM ${BASE_IMAGE_BUILD} AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY app /app
RUN cargo chef prepare --recipe-path /app/recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json /app/recipe.json
RUN cargo chef cook --release --recipe-path /app/recipe.json

COPY app /app
RUN cargo install --path /app

FROM ${BASE_IMAGE_RUN} AS runner
COPY --from=builder /usr/local/cargo/bin/app /app

CMD ["/app"]