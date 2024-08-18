_list:
    @just --list

msrv := ```
    cargo metadata --format-version=1 \
    | jq -r 'first(.packages[] | select(.source == null and .rust_version)) | .rust_version' \
    | sed -E 's/^1\.([0-9]{2})$/1\.\1\.0/'
```
msrv_rustup := "+" + msrv

# Check project.
[group("lint")]
check: && clippy
    just --unstable --fmt --check
    fd --hidden -e=toml --exec-batch taplo format --check
    fd --hidden -e=toml --exec-batch taplo lint
    fd --hidden --type=file -e=md -e=yml --exec-batch prettier --check
    cargo +nightly fmt -- --check

# Format project.
[group("lint")]
fmt: update-readmes
    just --unstable --fmt
    fd --hidden -e=toml --exec-batch taplo format
    fd --hidden --type=file -e=md -e=yml --exec-batch prettier --write
    cargo +nightly fmt

# Update READMEs from crate root documentation.
[group("lint")]
update-readmes:
    cargo rdme --force

# Lint workspace with Clippy.
[group("lint")]
clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --no-default-features --all-features
    cargo hack --feature-powerset clippy --workspace

# Downgrade dev-dependencies necessary to run MSRV checks/tests.
[private]
downgrade-msrv:
    @ echo "No downgrades currently necessary"

# Test workspace using MSRV.
[group("test")]
test-msrv: downgrade-msrv (test msrv_rustup)

# Test workspace.
[group("test")]
test toolchain="":
    cargo {{ toolchain }} nextest run --workspace --no-default-features
    cargo {{ toolchain }} nextest run --workspace --all-features
    cargo {{ toolchain }} test --doc --workspace --all-features
    RUSTDOCFLAGS="-D warnings" cargo {{ toolchain }} doc --workspace --no-deps --all-features
