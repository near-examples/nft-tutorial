#!/bin/bash

cd onchain_tests
bash arch.sh && yarn install
yarn run test
cd ..