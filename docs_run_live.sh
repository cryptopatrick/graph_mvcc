# https://benjamincongdon.me/blog/2018/08/22/Live-Refreshing-Cargo-Docs/
cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'
