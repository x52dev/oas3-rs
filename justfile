_list:
    @just --list

toolchain := ""
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
    cargo rdme --workspace-project=oas3 --readme-path=crates/oas3/README.md --force
    cargo rdme --workspace-project=roast --readme-path=crates/roast/README.md --force

# Lint workspace with Clippy.
[group("lint")]
clippy:
    cargo clippy --workspace --all-targets --no-default-features
    cargo clippy --workspace --all-targets --all-features
    cargo hack --feature-powerset clippy --workspace --all-targets

# Downgrade dev-dependencies necessary to run MSRV checks/tests.
[private]
downgrade-for-msrv:
    cargo {{ toolchain }} update -p=backtrace --precise=0.3.74 # next ver: 1.82.0
    cargo {{ toolchain }} update -p=idna_adapter --precise=1.2.0 # next ver: 1.82.0
    cargo {{ toolchain }} update -p=litemap --precise=0.7.4 # next ver: 1.81.0
    cargo {{ toolchain }} update -p=zerofrom --precise=0.1.5 # next ver: 1.81.0

# Test workspace using MSRV.
[group("test")]
test-msrv:
    @just toolchain={{ msrv_rustup }} downgrade-for-msrv
    @just toolchain={{ msrv_rustup }} test

# Test workspace.
[group("test")]
test toolchain="":
    cargo {{ toolchain }} nextest run --workspace --no-default-features
    cargo {{ toolchain }} nextest run --workspace --all-features
    cargo {{ toolchain }} test --doc --workspace --all-features
    RUSTDOCFLAGS="--cfg=docsrs -D warnings" cargo {{ toolchain }} doc --workspace --no-deps --all-features

# Test workspace and generate Codecov coverage file
test-coverage-codecov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --codecov --output-path codecov.json

# Test workspace and generate LCOV coverage file
test-coverage-lcov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --lcov --output-path lcov.info

# Document crates in workspace.
doc *args:
    RUSTDOCFLAGS="--cfg=docsrs -D warnings" cargo +nightly doc --workspace --all-features {{ args }}
