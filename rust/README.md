## Run test coverage

```sh
cargo llvm-cov --ignore-filename-regex=".*_tests.rs" --html --open
```

## Run macro expansion viewer

Install `cargo-expand` if you haven't already:

```sh
cargo install cargo-expand
```

Then run the following command to view the macro expansions for the `green::macros` module's tests:

```sh
cargo expand -p syntax --lib green::macros --tests
```