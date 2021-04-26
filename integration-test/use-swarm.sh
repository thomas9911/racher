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
