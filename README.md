This is a trading platform of Pokemon Card.

- Support RESTful API (with [OpenAPI Specification](./design/openapi.yml))
- Storage with PostgreSQL
- [Containerize](./dockerfile)
- Support GraphQL query
- CI/CD with Github Action and AWS ECS [link](./.github/workflows/)
- [Docker Compose](./docker-compose.yml) 
- Support Log aggreagation to loki, Grafana

# Dev setup
- need to install postgres client
- Prepare `.env` file:
```
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgres://postgres:postgrespw@localhost:49153/pokemon
RUST_LOG=sqlx=error,info
RUST_BACKTRACE=0
```

- Setup Postgres database
```
sqlx database create
sqlx migrate run
```

- If any Postgres query changed, please execute the following command before commit
```
cargo sqlx prepare
```

# Deploy
## Centralized Logging
- loki-docker-driver + loki + Grafana
- execute command
```
docker plugin install grafana/loki-docker-driver:latest --alias loki --grant-all-permissions
docker compose -f docker-compose-log.yml up
```
- Setup data source in Grafana with loki (http://loki:3100)

## Service
```
docker compose up
```