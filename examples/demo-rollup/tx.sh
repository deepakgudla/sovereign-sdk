#!/bin/bash

make test-create-token
cargo run --bin sov-cli -- transactions import from-file bank --chain-id 0 --path ../test-data/requests/transfer.json
cargo run --bin sov-cli rpc submit-batch by-address sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94

wait
