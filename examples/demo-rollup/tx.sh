#!/bin/bash

# make test-create-token
cargo run --bin sov-cli -- transactions import from-file bank --chain-id 0 --path ../test-data/requests/transfer.json
cargo run --bin sov-cli rpc submit-batch by-address sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94



# curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ledger_getBatches","params":[["0x322cf0811fd40d97a201a282da75020027e3a07b7fe32568e9680de9035395eb"]],"id":1}' http://127.0.0.1:12345


35b2e39503f9da25badb9259d1d9f9d8c061333b06448e325eba063c90d3ae35


0x8629352753e9423297fa479b88b4bdc0d09cf56448bdd2d391c2d542de417f70
# curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"bank_supplyOf","params":["sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94"],"id":1}' http://127.0.0.1:12345


# curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ledger_getBatches","params":[["0x8629352753e9423297fa479b88b4bdc0d09cf56448bdd2d391c2d542de417f70"]],"id":1}' http://127.0.0.1:12345


# curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[{ "batch_id": 1, "offset": 0}]],"id":1}' http://127.0.0.1:12345













