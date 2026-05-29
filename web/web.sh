set -x
pkill -f "eda4-web-server" 2>/dev/null; sleep 1
cargo run -p eda4-web-server
npx playwright test
# cargo test -p eda4-web-server