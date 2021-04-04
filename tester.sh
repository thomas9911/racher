curl -X POST 127.0.0.1:9226/get/testing

echo

curl -X POST 127.0.0.1:9226/get/one

echo

curl -X POST -H "Content-Type: application/json" -d '{"key1":"value1", "key2":"value2"}' 127.0.0.1:9226/set/one

echo

curl -X POST 127.0.0.1:9226/get/one

echo

curl -X POST 127.0.0.1:9226/keys

echo

curl -X POST 127.0.0.1:9226/del/one

echo

curl -X POST 127.0.0.1:9226/del/one

echo
