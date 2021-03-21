# chacha20
A simple chacha20_poly1305 CLI encryption tool

## Building
Requires Rust and Cargo to build.
Run `cargo build --release`, the binary will be built to `./target/release/chacha20`.

### Testing
Run `cargo test && cargo build && ./test.sh debug` to test the program.
Alternatively, run `./test.sh` after building to test the release build's correctness.

# Usage
Copies stdin to stdout while encrypting or decrypting with the stream cipher.

## Modes
* Encrypt - Encrypt stdin to stdout
* Decrypt - Decrypt stdin to stdout
* Keygen - Generate a random key and IV and print them to stdout

To see a more detailed explenation run `chacha20` with no arguments.

## Example

```shell
$ echo "Hello world!" | chacha20 e 2>keys.cck > output.cc20
$ chacha20 d $(cat keys.cck) < output.cc20
Hello world!
```

# License
GPL'd with <3
