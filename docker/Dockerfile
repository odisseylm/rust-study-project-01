
# Build the dependencies without the actual source code to cache dependencies separately
RUN cargo build --release

# Now copy the source code
COPY ./src ./src

# Build your application
RUN cargo build --release

# Start a new stage to create a smaller image without unnecessary build dependencies
#FROM debian:buster-slim
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/app/target/release/testing./ ???

# Command to run the application
CMD ["./testing"]
