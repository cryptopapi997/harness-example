# harness-example

## Setup

To run this project, you need to:

1. Add your authentication token to `.cargo/config.toml`
2. Install the Arcium CLI via https://docs.arcium.com/developers

## Building Circuits

Build the circuits with:
```bash
arcium build --skip-keys-sync --no-program
```

## Running

Once the circuit is built, run the project:
```bash
cd runner
cargo run
```