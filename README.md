# Sumcheck
Implementation of sumcheck protocol in Rust

Overview

This Rust project implements the Polynomial Sumcheck Protocol, a core building block in interactive proofs used within areas such as zero-knowledge proofs and complexity theory. The protocol checks whether a multivariate polynomial's sum over all possible Boolean inputs equals a claimed value, enabling verification of complex computations with minimal communication.

Features

Polynomial Input: Allows users to define and input multivariate polynomials.
Prover and Verifier Overrides: Users can manually override the polynomial values for the prover and verifier, offering flexibility for different testing and debugging scenarios.
Interactive Proof Simulation: Simulates the interaction between a prover and verifier, executing the sumcheck protocol step-by-step.
Validation: Automatically checks polynomial properties (like univariate conditions) and ensures the protocol steps follow the expected sequence, providing detailed feedback on any protocol violations.