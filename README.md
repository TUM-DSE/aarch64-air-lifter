# AArch64 to AIR lifter

## Getting started

To get started, install Rust (https://rustup.sh) and the following dependencies:

```shell
cargo install just cargo-sort cargo-nextest
```

All useful commands can be found in the top-level [Justfile](./Justfile) (Similar to a Makefile).

Run the following commands to get started:

```shell
# format the code
just fmt
# run tests
just test
# run lints
just check
```

### Pre-commit

Create a `.git/hooks/pre-commit` to run checks automatically every time you commit:

```shell
cat > .git/hooks/pre-commit << EOF
#!/usr/bin/env bash

set -euo pipefail

just lint
just test
EOF
chmod +x .git/hooks/pre-commit
```


