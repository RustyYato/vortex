part1:
    cargo build --release
    ./maelstrom/maelstrom test -w echo --bin ./target/release/echo --node-count 1 --time-limit 10
part2:
    cargo build --release
    ./maelstrom/maelstrom test -w unique-ids --bin ./target/release/generate --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
part3a:
    cargo build --release
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/broadcast --node-count 1 --time-limit 20 --rate 10
part3b:
    cargo build --release
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/multi-node-broadcast --node-count 5 --time-limit 20 --rate 10
part3c:
    cargo build --release
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/multi-node-partition --node-count 5 --time-limit 20 --rate 10 --nemesis partition
part3d *args:
    cargo build --release
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/multi-node-performance-1 --node-count 25 --time-limit 20 --rate 100 --latency 100 {{args}}
part3e *args:
    cargo build --release
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/multi-node-performance-2 --node-count 25 --time-limit 20 --rate 100 --latency 100
