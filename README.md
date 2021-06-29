# cryptolab-web-server

## Build and setup

1. Run `cargo build --release`

2. Create ```config.json``` in ```/config``` folder
and paste the following content to the .json file.

```json=

    "db_address": "127.0.0.1",
    "db_port": 27017,
    "kusama_db_name": "kusama",
    "polkadot_db_name": "polkadot",
    "db_has_credential": false,
    "db_username": "",
    "db_password": "",

    "port": 3030,
    "cors_url": ["http://127.0.0.1:3030", "http://localhost:3030", "http://127.0.0.1:8080"],

    "new_cache_folder": "../chain-data-collector/cache/kusama",
    "new_cache_folder_polkadot": "..//chain-data-collector/cache/polkadot",

    "staking_rewards_collector_dir": "../staking-rewards-collector",
    "serve_www": true
}
```

`db_address` and `db_port` indicate the address of a mongoDB server.

`kusama_db_name` and `polkadot_db_name` are the name of DB to save each data.

`db_has_credential` indicates whether the service needs to include the username/password while connecting to the DB

`db_username` and `db_password` are the credential used for login the DB if `db_has_credential` is set to true

`port` is the http server of this service

`cors_url` is an array of url which the server should allow cross origin.

`new_cache_folder` and `new_cache_folder_polkadot` should be assigned to the folders where `chain-data-collector` saves cache files for each chain.

`staking_rewards_collector_dir` should be assigned to the folder where `Staking rewards collector` resides in.

`serve_www` indicates whether the front end static files are served in this service.

## Test

Run `cargo test`

## Run

