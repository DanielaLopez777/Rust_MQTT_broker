#!/bin/bash

NUM_PUB=$1
NUM_SUB=$2
PAYLOAD=$3
EXEC_TIME=$4
FREQ=$5

CLIENT_PATH="../C_Mosquitto"
CLIENT_BIN="$CLIENT_PATH/client"

echo "Compiling..."
gcc "$CLIENT_PATH/client.c" -o "$CLIENT_BIN" -lmosquitto

# ================================
# Guardar PIDs
# ================================
SUB_PIDS=()
PUB_PIDS=()

echo "Starting subscribers..."

for ((i=0;i<NUM_SUB;i++))
do
    "$CLIENT_BIN" sub $i &
    SUB_PIDS+=($!)
done

sleep 2

echo "Starting publishers..."

for ((i=0;i<NUM_PUB;i++))
do
    "$CLIENT_BIN" pub $i "$PAYLOAD" "$EXEC_TIME" "$FREQ" &
    PUB_PIDS+=($!)
done

# ================================
# Esperar a que terminen publishers
# ================================
echo "Waiting publishers to finish..."
for pid in "${PUB_PIDS[@]}"
do
    wait $pid
done

echo "Publishers finished."

# ================================
# Terminar subscribers
# ================================
echo "Stopping subscribers..."
for pid in "${SUB_PIDS[@]}"
do
    kill $pid 2>/dev/null
done

wait

echo "Test finished successfully."