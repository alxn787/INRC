Decentralized CDP Protocol on Solana
A decentralized borrowing protocol built on the Solana blockchain that allows users to deposit USDC as collateral to mint INRC, a synthetic stablecoin pegged to the Indian Rupee. The protocol is secured by a system of over-collateralization and automated liquidations, with price data supplied by the Pyth Network.


Overview
This project implements a Collateralized Debt Position (CDP) model, a fundamental building block of decentralized finance (DeFi). It enables users to take out loans in the form of a synthetic asset (INRC) by locking up (USDC) as collateral.

The protocol ensures system solvency by requiring that all debt be over-collateralized. If the value of a user's collateral drops below a specified threshold, their position becomes eligible for liquidation by other users, who are incentivized with a bonus.

Key Features
Mint Stablecoins: Deposit USDC into a secure, common treasury to mint INRC tokens.

Withdraw Collateral: Repay your INRC debt to unlock and withdraw your USDC collateral.

On-Chain Liquidations: A transparent and automated liquidation mechanism protects the protocol from insolvency.

Pyth Oracle Integration: Utilizes Pyth Network's high-fidelity price feeds for accurate and real-time valuation of collateral.


Getting Started
Prerequisites
Rust: Install Rust

Solana Tool Suite: Install Solana CLI

Anchor Framework: Install Anchor

Node.js & Yarn: Required for running tests.

Installation & Setup
Clone the repository:

git clone <https://github.com/alxn787/INRC>
cd <INRC/contract-inrc>

Build the program:

anchor build

This will compile the Rust code and generate the program's IDL (Interface Definition Language).

Run tests:

anchor test

This command will run the integration tests located in the tests/ directory against a local validator.
