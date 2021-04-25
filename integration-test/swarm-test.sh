export RACHER_BACKUP_SKIP_LOADING=1
export RACHER_BACKUP_AMOUNT=0
export RACHER_BACKUP_INTERVAL=120
# export RACHER_LOGGER_LEVEL=DEBUG

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

curl -X POST 127.0.0.1:9227/get/one

echo

curl -X POST -H "Content-Type: application/json" -d '{"key1":"value1", "key2":"value2"}' 127.0.0.1:9226/set/one

echo

curl -X POST 127.0.0.1:9227/get/one

echo

curl -X POST 127.0.0.1:9228/get/one

echo

curl -X POST -H "Content-Type: application/json" -d '{"other":"some"}' 127.0.0.1:9228/set/one

echo

sleep 1

echo

curl -X POST 127.0.0.1:9226/get/one

echo

curl -X POST 127.0.0.1:9227/get/one

echo

curl -X POST 127.0.0.1:9228/get/one

echo

curl -X POST 127.0.0.1:9229/get/one

echo

curl -X POST 127.0.0.1:9230/get/one

echo

kill $process1
kill $process2
kill $process3
kill $process4
kill $process5
