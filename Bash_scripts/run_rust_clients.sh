#!/bin/bash

NUM_PUB=$1
NUM_SUB=$2
PAYLOAD=$3
EXEC_TIME=$4
FREQ=$5

echo "Building Rust project..."
cargo build --release

CLIENT_BIN="../target/release/client"

SUB_PIDS=()
PUB_PIDS=()

echo "Starting subscribers..."

for ((i=0;i<NUM_SUB;i++))
do
    "$CLIENT_BIN" sub &
    SUB_PIDS+=($!)
done

sleep 2

echo "Starting publishers..."

for ((i=0;i<NUM_PUB;i++))
do
    "$CLIENT_BIN" pub "$PAYLOAD" "$EXEC_TIME" "$FREQ" &
    PUB_PIDS+=($!)
done

echo "Waiting publishers..."

for pid in "${PUB_PIDS[@]}"
do
    wait $pid
done

echo "Stopping subscribers..."

for pid in "${SUB_PIDS[@]}"
do
    kill $pid 2>/dev/null
done

wait

echo "Test finished successfully."