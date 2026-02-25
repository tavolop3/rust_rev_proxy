#!/bin/bash

# Number of times we are going to run oha
ITERATIONS=5
# Duration of each test
DURATION="10s"
CONNECTIONS=100

echo "Compiling everything in release mode..."
cargo build --release

# If you have the dummy server as a separate binary, make sure the path is correct.
# Assuming you placed it in src/bin/dummy_server_http.rs
echo "Starting the dummy server..."
./../target/release/dummy_server_http &
DUMMY_PID=$!

# Give the OS a second to assign ports
sleep 1

echo "Starting the Reverse Proxy..."
./../target/release/proxy &
PROXY_PID=$!

sleep 1

echo "----------------------------------------"
echo "Starting $ITERATIONS benchmark rounds..."
echo "----------------------------------------"

TOTAL_RPS=0

for i in $(seq 1 $ITERATIONS); do
    echo -n "Round $i... "
    
    # Run oha, filter the "Requests/sec" line, and extract the second field (the number)
    RPS=$(oha -c $CONNECTIONS -z $DURATION http://127.0.0.1:8080 | grep "Requests/sec" | awk '{print $2}')
    
    echo "$RPS req/sec"
    
    # Add to total (use 'bc' because Bash does not handle decimals natively)
    TOTAL_RPS=$(echo "$TOTAL_RPS + $RPS" | bc)
    
    # 2-second pause between tests to let TIME_WAIT sockets breathe a bit
    sleep 2
done

# Calculate average
AVERAGE=$(echo "scale=2; $TOTAL_RPS / $ITERATIONS" | bc)

echo "========================================"
echo "AVERAGE REQUESTS/SEC: $AVERAGE"
echo "========================================"

echo "Killing background processes..."
kill $PROXY_PID
kill $DUMMY_PID

echo "Done."
