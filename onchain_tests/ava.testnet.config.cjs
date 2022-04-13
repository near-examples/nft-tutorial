module.exports = {
  ...require('near-workspaces-ava/ava.testnet.config.cjs'),
  ...require('./ava.config.cjs'),
};

// Add files you only want to run in Sandbox mode here
module.exports.files.push(
  '!__tests__/test_contract*',   // reducing tests run in testnet to only XCC tests avoids Rejected promise errors
);
