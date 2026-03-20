Set-StrictMode -Version Latest
Write-Output "Running cargo fmt..."
cargo fmt
Write-Output "Building release..."
cargo build --release
Write-Output "Running tests..."
cargo test --all --verbose
