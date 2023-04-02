## Launch

Run the master node:
```
cargo run -- -a 0.0.0.0:3000 --rpc_url http://localhost:8545 --contract '0x..'
```
repeat for every peer like this:
```
cargo run -- -a 0.0.0.0:3001 --peer 127.0.0.1:3000 --rpc_url http://localhost:8545 --contract '0x..'
cargo run -- -a 0.0.0.0:3002 --peer 127.0.0.1:3000 --rpc_url http://localhost:8545 --contract '0x..'
cargo run -- -a 0.0.0.0:3003 --peer 127.0.0.1:3000 --rpc_url http://localhost:8545 --contract '0x..'
```

## API
```
GET /data/:address - Get the whole data set
POST /data/:address - Set data: encode, chunk, and send to peers
GET /data/:address/partial - Get partial data
POST /data/:address/partial - Set partial data
```