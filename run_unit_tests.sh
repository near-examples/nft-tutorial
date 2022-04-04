#!/bin/bash

cd market-contract
cargo test -- --nocapture
cd ..
cd nft-contract
cargo test -- --nocapture