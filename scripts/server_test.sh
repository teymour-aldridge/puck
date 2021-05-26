#!/usr/bin/env bash
# Author michael <themichaeleden@gmail.com>
set -euo pipefail
set -x

function cleanup() {
    kill -9 ${WSSERVER_PID}
}
trap cleanup TERM EXIT

function test_diff() {
    if ! diff -q \
        <(jq -S 'del(."Puck" | .. | .duration?)' 'autobahn/server-results.json') \
        <(jq -S 'del(."Puck" | .. | .duration?)' 'autobahn/server/index.json')
    then
        echo 'Difference in results, either this is a regression or' \
             'autobahn/expected-results.json needs to be updated with the new results.'
        exit 64
    fi
}

cargo build --release --bin echo

cargo run --release --bin echo & WSSERVER_PID=$!

docker run --rm \
    -v "${PWD}/autobahn:/autobahn" \
    --p 5051:5051 \
    crossbario/autobahn-testsuite \
    wstest -m fuzzingclient -s './autobahn/fuzzingclient.json'

test_diff
