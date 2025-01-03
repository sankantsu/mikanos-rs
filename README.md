# mikanos-rs

Rust implementation of educational OS [MikanOS](https://github.com/uchan-nos/mikanos).

- mikanos-rs-loader: A UEFI bootloader for mikanos-rs.
- mikanos-rs-kernel: The mikanos-rs kernel.

# Requirements

- Requires [Rust 2024 Edition](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

# Setup

1. Install a nightly version of Rust toolchain.
```shell
$ rustup toolchain install nightly
```

2. Override the Rust toolchain used in this project with a nightly.
```shell
$ rustup override set nightly
```

3. run `rustup target add x86_64-unknown-uefi`.
```shell
$ rustup target add x86_64-unknown-uefi
```

# Running

1. run start script.
```shell
$ bash run.sh
```

# References

- [How Rust is Made and “Nightly Rust”](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)

