check: lint test

# Bump all deps, including incompatible version upgrades
bump:
    just ensure_installed upgrade
    cargo update
    cargo upgrade --incompatible
    just test

lint:
    just ensure_installed sort
    cargo fmt --all -- --check
    cargo sort --workspace --check
    cargo clippy --tests --benches --workspace -- -D warnings

test:
    just ensure_installed nextest
    cargo nextest run --workspace

alias fmt := format
format:
    just ensure_installed sort
    cargo fmt
    cargo sort --workspace

ensure_installed *args:
    #!/bin/bash
    cargo --list | grep -q {{ args }}
    if [[ $? -ne 0 ]]; then
        echo "error: cargo-{{ args }} is not installed. Install it with `cargo install cargo-{{ args }}`"
        exit 1
    fi
