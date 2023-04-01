## Launch
```
cargo run -- -a 0.0.0.0:3000 \
    -p localhost:3001 \
    -p localhost:3002 \
    -p localhost:3003 \
```
repeat for every peer like this:
```
cargo run -- -a 0.0.0.0:3001 \
    -p localhost:3000 \
    -p localhost:3002 \
    -p localhost:3003 \
```
etc

Not ideal to implement p2p this way, but it works for now.