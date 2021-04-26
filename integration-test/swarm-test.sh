export RACHER_BACKUP_SKIP_LOADING=1
export RACHER_BACKUP_AMOUNT=0
export RACHER_BACKUP_INTERVAL=120
# export RACHER_LOGGER_LEVEL=DEBUG

function get() {
    curl -X POST "127.0.0.1:$1/get/one"
    echo
}

function update() {
    curl -X POST -H "Content-Type: application/json" -d "$2" "127.0.0.1:$1/set/one"
    echo
}

function delete() {
    curl -X POST "127.0.0.1:$1/del/one"
    echo
}

cargo build

cargo run -- -a 127.0.0.1:9226 &
process1=$!

sleep 3

cargo run -- join -a 127.0.0.1:9227 -j http://127.0.0.1:9226 &
process2=$!
cargo run -- join -a 127.0.0.1:9228 -j http://127.0.0.1:9226 &
process3=$!
cargo run -- join -a 127.0.0.1:9229 -j http://127.0.0.1:9226 &
process4=$!
cargo run -- join -a 127.0.0.1:9230 -j http://127.0.0.1:9226 &
process5=$!

sleep 5

for i in $(seq 1 12); do

    for port in $(seq 9226 9230); do
        get $port
    done

    echo

    update 9226 '{"key1":"value1", "key2":"value2"}'

    for port in $(seq 9226 9230); do
        get $port
    done

    echo

    update 9229 '{"other":"some"}'

    for port in $(seq 9226 9230); do
        get $port
    done

    echo

    delete 9230

    for port in $(seq 9226 9230); do
        get $port
    done

    echo

    echo "=====> next"

    echo

done

kill $process1
kill $process2
kill $process3
kill $process4
kill $process5
