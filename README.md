# CloudEvent Diary (CED)

This is a simple app to log CloudEvents into a Postgres table.

## Set Environment

This app needs a Postgres server for testing, which run when skaffold-building. Fill [./app/.sample_env](./app/.sample_env) and rename it to [./app/.env](./app/.env).

## Run

### Local

You can develop, build, and run this Rust project as you normally would:

```bash
cd app
cargo run
```

### Cluster

Assuming you have set up your K8S cluster, Knative, and Skaffold environment correctly, you can:

```bash
skaffold dev
```

## Test

```bash
cd app && cargo test
```

### Local with curl

Run

```bash
cd app && cargo run
```

then

```bash
curl localhost:8080 -v   -H "Content-Type: application/cloudevents+json"   -d '{
        "specversion": "1.0",
        "type": "com.acme.events.something",
        "source": "com.acme.apps.ingress",
        "id": "370058fc-0d71-11ee-be56-0242ac120002",
        "time": "2023-10-01T00:00:00Z",
        "something": "else",
        "datacontenttype": "application/json",
        "knativeerrorcode": 500,
        "knativeerrordest": "http://acme.com",
        "knativeerrordata": "",
        "data": {
          "body": "text",
          "volume": 90
        }
    }'
```
