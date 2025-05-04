Solana Liquidity Pool AMM
Overview
This project implements an Automated Market Maker (AMM) liquidity pool on the Solana blockchain, inspired by Raydium. Written in pure Rust, it allows users to initialize a pool, add/remove liquidity, and swap tokens using a constant product formula (x * y = k) with a 0.3% fee. The program is deployed on Solana's devnet and includes a Rust client for testing all instructions.
Features
InitializePool: Creates a new liquidity pool with a specified token pair (e.g., SOL/USDC).

AddLiquidity: Deposits tokens into the pool and mints liquidity tokens.

RemoveLiquidity: Burns liquidity tokens to withdraw proportional tokens.

Swap: Trades tokens with a constant product formula and 0.3% fee.

Security: Uses Program-Derived Addresses (PDAs) for authority, extensive account validation, and custom error handling.

Testing: Includes a Rust client to test all instructions on devnet with real SPL tokens.

Program Details
Program ID: 2SRp5ENH631KzuRGNXWcdLi59pnvKNNoTm37pMRcBH3Q

Deployment: Solana devnet

Language: Rust

Dependencies:
solana-program: 1.18.9

borsh: 0.10.3

borsh-derive: 0.10.3

Client Testing
The repository includes a client (solana-liq-pool-client) to test the program on devnet:
Setup: Creates SPL token mints, user accounts, vaults, and liquidity mint.

Tests: Executes InitializePool, AddLiquidity, RemoveLiquidity, and Swap instructions, verifying pool state after each.

Dependencies:
solana-client: 1.18.9

solana-sdk: 1.18.9

solana-program: 1.18.9

borsh: 0.10.3

borsh-derive: 0.10.3

anyhow: 1.0.86

serde_json: 1.0.128

shellexpand: 3.0.0

Prerequisites
Rust (stable)

Solana CLI (solana --version should show 1.18.x)

Node.js (optional, for additional tooling)

A funded Solana keypair on devnet (~/.config/solana/id.json)

Installation
Clone the repository:
bash

git clone https://github.com/haryodeji11/solana-liq-pool.git
cd solana-liquidity-pool

Build the program:
bash

cargo build-bpf

Deploy to devnet:
bash

solana program deploy target/deploy/solana_liq_pool.so

Set up the client:
bash

cd solana-liq-pool-client
cargo build

Create SPL token accounts (see docs/testing.md for details).

Usage
Configure Solana CLI for devnet:
bash

solana config set --url https://api.devnet.solana.com

Run the client tests:
bash

cd solana-liq-pool-client
cargo run

Verify transaction signatures and pool state in the console output.

Security Considerations
Rug Pull Prevention: Uses PDA authority for vault transfers, ensuring only the program can move funds.

Honeypot Mitigation: Tested with trusted SPL tokens created by the developer.

Auditing: Recommended before mainnet deployment (e.g., via Certik or Hacken).

Future Improvements
Add slippage protection for swaps.

Implement fee distribution for liquidity providers.

Support program upgrades using BPFLoaderUpgradeable.

Contributing
Contributions are welcome! Please submit issues or pull requests to enhance functionality or fix bugs.
License
This project is licensed under the MIT License. See the LICENSE file for details.
Acknowledgments
Inspired by Raydium’s AMM model.

Built with the Solana Rust SDK and SPL Token program.

