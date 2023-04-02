# Shared Storage

One of the problems of current blockchain scalability is that the blockchain size is growing very fast. The computation performance is already efficiently scaled by recursive zk rollups and VMs. But if you perform billions and trillions of transactions, you should store the state of blockchain somewhere.

The idea of decentralized zkSNARK-driven storage is inspired by Filecoin. BTW, there are no strong guarantees that the data will not be lost. 

Our idea is based on the following observation:

If you have a N-sized vector, you can represent it as a polynomial of degree N-1. Then if you extrapolate it to 2N-sized evaluation domain, any N points are enough to reconstruct the original vector.

This is the main idea of our storage. If we split 2N points into M chunks and store them on M different nodes, we can reconstruct the original vector, if at least 50% of nodes are honest.

## Demo

https://youtu.be/CaJYXCTBzWk

## Contacts

https://twitter.com/ZeroPoolNetwork
https://zeropool.network
