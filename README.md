# Racher

HTTP cache

## API

all endpoints should be "POST"ed to.

### /get/:name

Get data under :name

### /set/:name

Set data under :name

### /del/:name

Delete data under :name

### /keys

Return a list of all keys:

```json
{"keys": ["key1", ...] }
```

### /purge

Deletes the whole cache

### /ping

Just returns `{"pong": true}`

### /\_internal

Internal api probably not what you want to use, only when you know what you are doing

## examples

### set

```sh
curl -X POST -H "Content-Type: application/json" -d '{"key1":"value1", "key2":"value2"}' 127.0.0.1:9226/set/one
```

### get

```sh
curl -X POST 127.0.0.1:9226/get/one
```
