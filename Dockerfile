FROM debian:trixie-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc && \
    ln -s /root/.local/bin/yt-dlp /usr/local/bin/yt-dlp && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the built binary from the builder stage
COPY ./target/release/v3x-events .

# Expose port 3000
EXPOSE 3000

# Set the entrypoint command to run your application
ENTRYPOINT ["./v3x-events"]
