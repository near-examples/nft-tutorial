## On-Chain Simulation Tests

These tests use [near-workspaces-ava](https://github.com/near/workspaces-js/tree/main/packages/ava): delightful, deterministic local testing for NEAR smart contracts.

If running on an M1 Mac, you will need to run `arch -x86_64 zsh` to switch to an x86 terminal before running tests or installing packages.

You will need to install [NodeJS](https://nodejs.dev/). Then you can use the `scripts` defined in [package.json](./package.json):
```commandLine
npm run test
```
or 
```commandLine
yarn run test
```