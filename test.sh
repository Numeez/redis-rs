#!/usr/bin/env bash
# Usage: start the server first, then run:  bash ttl_test.sh
# Override the client path if needed:       CLIENT=./target/release/client bash ttl_test.sh

C="${CLIENT:-./target/debug/client}"
OUT="ttl_results.txt"

: > "$OUT"

run() {
    local expect="$1"; shift
    {
        echo "\$ $*"
        echo "expected: $expect"
        echo -n "got:      "
        "$C" "$@" 2>&1 | tr '\n' ' '
        echo
        echo
    } >> "$OUT"
}

pause() {
    echo "--- sleep $1 ---" >> "$OUT"
    echo >> "$OUT"
    sleep "$1"
}

echo "=== 1. key expires on time ===" >> "$OUT"
run "nil (from set)"        set k hello
run "(integer) 1"           pexpire k 3000
run "about 3000"            pttl k
pause 4
run "(nil)"                 get k
run "(integer) -2"          pttl k

echo "=== 2. extended TTL: stale entry skipped ===" >> "$OUT"
run "nil (from set)"        set k hello
run "(integer) 1"           pexpire k 1000
run "(integer) 1"           pexpire k 60000
pause 2
run "hello"                 get k
run "about 58000"           pttl k

echo "=== 3. set clears the TTL ===" >> "$OUT"
run "(integer) 1"           pexpire k 1000
run "nil (from set)"        set k world
pause 2
run "world"                 get k
run "(integer) -1"          pttl k

echo "=== 4. whole sorted set expires ===" >> "$OUT"
run "(integer) 1"           zadd board 1 a
run "(integer) 1"           pexpire board 1000
pause 2
run "(nil)"                 zscore board a

echo "=== 5. pexpire on missing key ===" >> "$OUT"
run "(integer) 0"           pexpire nosuchkey 1000
run "(integer) -2"          pttl nosuchkey

echo "=== 6. negative ms rejected ===" >> "$OUT"
run "(error ...)"           pexpire k -100

echo "done, results in $OUT"
