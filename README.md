# Wordle Solver (SIMD Branch)

A command-line Wordle assistant and solver written in Rust. This specific branch (`simd`) optimizes the word matching, entropy calculations, and filtering using Single Instruction, Multiple Data (SIMD) and bitwise operations.

## Features

- **SIMD Optimization**: Uses `#![feature(portable_simd)]` and `std::simd` to accelerate the Information Theory (entropy) calculations.
- Suggests the best next guesses by mathematically determining which word eliminates the most possibilities.
- Ranks words by green matches, included letters, excluded letters, and rejected words.
- Interactive CLI: input your guess and the feedback received (green, yellow, black), and it dynamically updates the suggestions.
- Fast execution utilizing Rust's nightly SIMD capabilities.

## Prerequisites

- [Rust / Cargo](https://rustup.rs/)
- **Nightly Rust Toolchain**: Because portable SIMD is currently an experimental feature in Rust, this branch requires the nightly compiler. 

To set your directory to use nightly Rust:
```bash
rustup override set nightly
```

## Setup

The program uses a list of 5-letter words (`sgb-words.txt`). This file should already be in the repository. If it is missing, you can download it by running:

```bash
make sgb-words.txt
```

## Usage

Run the program using Cargo in release mode (crucial for maximizing the SIMD performance gains):

```bash
cargo run --release
```

Once running, the interactive prompt will guide you:

1. **What is your guess:** Enter the 5-letter word you guessed.
2. **What are colours of match:** Enter the 5-character result from Wordle using `g` (green), `y` (yellow), or `b` (black/gray).

### Example
If you guessed `stare` and the `s` is green, the `a` is yellow, and the rest are gray/black, you would input:

```text
What is your guess
stare
What are colours of match (g)reen (y)ellow or (b)lack
gbbyb
```

The program will then display the top recommended words based on entropy and other metrics to help you make your next guess.

## Benchmarking

This project includes benchmarks using `criterion` to compare the standard scoring against the SIMD optimizations. 

To run the benchmarks:

```bash
cargo bench
```
