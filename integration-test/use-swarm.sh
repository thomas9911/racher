curl -X POST 127.0.0.1:9227/get/one

echo

curl -X POST -H "Content-Type: application/json" -d '{"key1":"value1", "key2":"value2"}' 127.0.0.1:9226/set/one

echo

curl -X POST 127.0.0.1:9227/get/one

echo

curl -X POST 127.0.0.1:9228/get/one

echo

curl -X POST -H "Content-Type: application/json" -d '{"other":"some"}' 127.0.0.1:9229/set/one

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
