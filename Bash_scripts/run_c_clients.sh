#!/bin/bash

# ================================
# Uso:
# ./run_c_clients.sh num_pub num_sub payload execution_time frequency
# ================================

NUM_PUB=$1
NUM_SUB=$2
PAYLOAD=$3
EXEC_TIME=$4
FREQ=$5

CLIENT_PATH="../C_Mosquitto"
CLIENT_BIN="$CLIENT_PATH/client"

# ================================
# Compilar cliente C
# ================================
echo "Compiling C client..."
gcc "$CLIENT_PATH/client.c" -o "$CLIENT_BIN" -lmosquitto

# ================================
# Lanzar Subscribers
# ================================
echo "Starting $NUM_SUB subscribers..."

for ((i=0;i<NUM_SUB;i++))
do
    "$CLIENT_BIN" sub &
    echo "Subscriber $i PID: $!"
done

# ================================
# Lanzar Publishers
# ================================
echo "Starting $NUM_PUB publishers..."

for ((i=0;i<NUM_PUB;i++))
do
    "$CLIENT_BIN" pub "$PAYLOAD" "$EXEC_TIME" "$FREQ" &
    echo "Publisher $i PID: $!"
done

wait
