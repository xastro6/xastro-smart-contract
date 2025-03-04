You're right to point that out! The command `cargo build-bpf` only needs to be mentioned once, as it's the primary command for building your Solana program. The duplicate `cargo build` should not be there.

Here's the corrected version:

```
# 🚀 WAGUS Reward System

A Solana smart contract (program) that powers the **WAGUS token** reward system. This program is designed to handle token-based rewards on the Solana blockchain, utilizing SPL tokens and built with Rust.

## 📜 Description
This project implements a decentralized reward system for the **WAGUS** ecosystem. Users can earn and claim rewards in WAGUS tokens through on-chain interactions.

## 🛠️ Built With
- [Rust](https://www.rust-lang.org/)
- [Solana Program Library (SPL)](https://spl.solana.com/)
- [Borsh](https://borsh.io/) (for serialization)
- Solana SDK

## 📂 Project Structure
```
├── src/                    # Program source code
├── tests/                  # Integration tests
├── Cargo.toml              # Rust dependencies and package info
├── target/deploy/          # Compiled Solana program (.so file)
└── README.md               # Project documentation
```

## 🚀 Deployments
| Network  | Program ID                              |
|----------|-----------------------------------------|
| Devnet   | `2ga161fxHesc8YATYz2CconNkTSpCJVABrjbBKGtRYGF` |

## ⚡ Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Node.js](https://nodejs.org/) (if using JS tests)
- Git

### Install Dependencies
```bash
cargo build-bpf
```

### Deploy Program
```bash
solana program deploy target/deploy/wagus_reward_system.so
```

## 🧪 Testing
```bash
cargo test
```

Or, for Solana test validator:
```bash
solana-test-validator
```

## 🌐 Resources
- [Solana Docs](https://docs.solana.com/)
- [SPL Token Docs](https://spl.solana.com/token)
- [Anchor Framework (optional)](https://book.anchor-lang.com/)

## 📜 License
This project is licensed under the [Apache 2.0 License](LICENSE).

## ✨ Author
**xastro**  
[GitHub](https://github.com/xastro6)
```
