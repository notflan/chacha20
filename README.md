# chacha20
A simple chacha20_poly1305 CLI encryption tool

## Building
Requires Rust and Cargo to build; also requires OpenSSL v1.1.0 or higher.
Run `cargo build --release`, the binary will be built to `./target/release/chacha20`.

### Testing
Run `cargo test && cargo build && ./test.sh debug` to test the program.
Alternatively, run `./test.sh` after building to test the release build's correctness.

# Usage
Copies stdin to stdout while encrypting or decrypting with the stream cipher `chacha20_poly1305`.

## Modes
* Encrypt - Encrypt stdin to stdout
* Decrypt - Decrypt stdin to stdout
* Keygen - Generate a random key and IV and print them to stdout

To see a more detailed explenation run `chacha20 help`.

## Formats
The key and IV is expected/generated in base64 format.
The key and IV sizes respectively are 32 and 12 bytes.

The ciphertext input and output is raw binary data. You can encode this to text formats if you want with whatever tool you choose (Example with `base64` below.)

## Example

Encrypting and decrypting a string to binary with randomly generated keys
```shell
$ echo "Hello world!" | chacha20 e 2>keys.cck > output.cc20
$ chacha20 d $(cat keys.cck) < output.cc20
Hello world!
```

The same but with text instead of binary ciphertexts

``` shell
$ echo "Hello world!" | chacha20 e 2>keys.cck | base64 > output.cc20.b64
$ base64 --decode output.cc20.b64 | chacha20 d $(cat keys.cck)
Hello world!
```

# License
GPL'd with <3
