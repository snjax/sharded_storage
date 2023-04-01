## Launch

Run the master node:
```
cargo run -- -a 0.0.0.0:3000
```
repeat for every peer like this:
```
cargo run -- -a 0.0.0.0:3001 --peer 127.0.0.1:3000
cargo run -- -a 0.0.0.0:3002 --peer 127.0.0.1:3000
cargo run -- -a 0.0.0.0:3003 --peer 127.0.0.1:3000
```
