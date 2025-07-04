#!/bin/bash
set -e

echo "Building release binary for x86_64-unknown-linux-musl..."
cargo build --release --target x86_64-unknown-linux-musl

echo "Creating output directory..."
rm -rf target/output
mkdir -p target/output/

echo "Copying tarball..."
cp -r tarball target/output/nginx-log-analyzer

echo "Copying built binary..."
cp target/x86_64-unknown-linux-musl/release/nginx-log-analyzer target/output/nginx-log-analyzer

echo "tar."
cd target/output/
tar czf nginx-log-analyzer.tgz nginx-log-analyzer

echo "All operations completed successfully."

