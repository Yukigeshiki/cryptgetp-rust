# cryptgetp-rust
Crypt-Get-P - a just for fun CLI tool to fetch cryptocurrency prices written in Rust.

### How to run:

First make sure you have Rust installed. To do this you can follow the instructions found [here](https://www.rust-lang.org/tools/install).

Clone the repo, cd into it and run:

```bash
cargo build --release
```

The pricing data is fetched from coinapi.io, so you'll need to get a free API key from [here](https://www.coinapi.io/pricing?apikey).

Once you have an API key you can run:

```bash
./target/release/cryptgetp -- BTC USD <your-key-here>
```

You can also run:

```bash
cargo run --release -- BTC USD <your-key-here>
```

if you'd prefer not to build the application first.

You get 100 free calls per day with your API key. More information about the API can be found [here](https://www.coinapi.io).
