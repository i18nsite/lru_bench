#!/bin/bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

./clippy.sh

cat <<'EOF'
Building benchmark...
EOF
cargo build --release --bench cache_comparison

cat <<'EOF'
Running benchmark (Compio Single Thread: Hashlink vs LRU vs Mini-Moka)...
Workload: Zipf distribution, 95% Read / 5% Write, ~90% Hit Rate Target
Backend Latency: Random 1-2ms
Enhanced warmup strategy enabled
EOF

# 运行 Criterion 测试
# 结果会输出到 target/criterion/report/index.html
cargo bench

# 获取 target 目录路径
TARGET_DIR=$(cargo metadata --no-deps --format-version 1 | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
SOURCE_REPORT_PATH="$TARGET_DIR/criterion"
SOURCE_CRITERION_PATH="$TARGET_DIR/criterion"

# 清理旧报告
rm -rf reports/criterion reports/html
mkdir -p reports

# 复制完整的 Criterion 报告到当前目录
cp -R "$SOURCE_REPORT_PATH" ./reports/

# 创建 HTML 符号链接以便于访问
ln -sf criterion/report ./reports/html

# 生成摘要报告
cat >./reports/summary.txt <<EOF
=== Benchmark Summary ===
Date: $(date)
Configuration:
- Cache Capacity: ${CACHE_CAPACITY:-7500}
- Total Keys: 10000
- Workload Size: 1000
- Zipf Parameter: 1.6
- Read Ratio: 95%
- Backend Latency: 1-2ms

EOF

# 如果有jq工具，提取详细结果
if command -v jq &>/dev/null; then
  echo "Detailed Results:" >>./reports/summary.txt
  # 从各个 estimates.json 文件提取结果
  for impl in hashlink_lru lru mini_moka_unsync; do
    ESTIMATE_FILE="./reports/criterion/Single-Thread Cache + Compio Async IO/$impl/new/estimates.json"
    if [ -f "$ESTIMATE_FILE" ]; then
      MEAN=$(jq -r '.mean.point_estimate' "$ESTIMATE_FILE" 2>/dev/null)
      UNIT=$(jq -r '.mean.unit' "$ESTIMATE_FILE" 2>/dev/null)
      if [ "$MEAN" != "null" ] && [ "$UNIT" != "null" ]; then
        # 转换为毫秒
        if [ "$UNIT" = "ns" ]; then
          MS=$(echo "scale=2; $MEAN / 1000000" | bc -l 2>/dev/null || echo "$MEAN")
          echo "$impl: ${MS} ms" >>./reports/summary.txt
        else
          echo "$impl: $MEAN $UNIT" >>./reports/summary.txt
        fi
      fi
    fi
  done
else
  echo "Install jq and bc for detailed results extraction" >>./reports/summary.txt
fi

cat <<EOF

Benchmark finished!
HTML report available at: ./reports/html/index.html
Full Criterion reports at: ./reports/criterion/
Summary saved to: ./reports/summary.txt
Original report location: $SOURCE_REPORT_PATH
EOF

# 如果是 macOS，尝试自动打开 HTML 报告
if [[ "$OSTYPE" == "darwin"* ]]; then
  open "./reports/html/index.html"
elif command -v xdg-open &>/dev/null; then
  xdg-open "./reports/html/index.html"
fi
