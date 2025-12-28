#!/bin/bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

echo "Running tests..."

# 运行 clippy 检查
echo "Running clippy checks..."
./clippy.sh

# 运行单元测试
echo "Running unit tests..."
cargo test --release

# 运行基准测试
echo "Running benchmarks..."
./run_bench.sh

echo "All tests completed successfully!"