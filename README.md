## About

I have been attempting to pass the Maelstrom distributed system challenges from [fly.io](https://fly.io/dist-sys/). These tests use Maelstrom, a
"workbench for learning distributed systems by writing your own", the Github of which can be found [here](https://github.com/jepsen-io/maelstrom).
I wrote the code in Rust, using Serde & serde_json. Other than that, only the Rust standard library was used. In order to test a binary with Maelstrom, follow the directions for installing Maelstrom found in either
link above. Once a binary is built, maelstrom is invoked with the path to the binary, as well as additional args to configure the test.

### Challenge #1: Echo

---

For challenge 1, the `node_runtime` fn should look like this:

```
node_runtime::<echo::EchoBody, echo::EchoNode>(node, tx, rx);
```

Challenge directions can be found [here](https://fly.io/dist-sys/1/).
The following test given in the fly.io directions passes:

```
./maelstrom test -w echo --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 10
```

### Challenge #2: Unique ID Generation

---

For challenge 2, the `node_runtime` fn should look like this:

```
node_runtime::<generate_id::GenerateGuidBody, generate_id::GenerateGuidNode>(node, tx, rx);
```

Challenge directions can be found [here](https://fly.io/dist-sys/2/).
The following test given in the fly.io directions passes:

```
 ./maelstrom test -w unique-ids --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```

### Challenge 3a: Single-Node Broadcast

---

For challenge 3, the `node_runtime` fn should look like this:

```
node_runtime::<broadcast::BroadcastBody, broadcast::BroadcastNode>(node, tx, rx);
```

Challenge directions can be found [here](https://fly.io/dist-sys/3a/).
The following test given in the fly.io directions passes:

```
./maelstrom test -w broadcast --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 20 --rate 10
```

### Challenge 4: Grow-Only Counter

---

For challenge 4, the `node_runtime` fn should look like this:

```
node_runtime::<grow_counter::CounterBody, grow_counter::CounterNode>(node, tx, rx);
```

Challenge directions can be found [here](https://fly.io/dist-sys/4/). While the directions instruct
the programmer to use a "sequentially-consistent key/value store service provided by Maelstrom", I could not
find any easy documentation on how to interact with this service, so I chose to simply send updates of each nodes
counters every second to the other nodes.
The following test given in the fly.io directions passes:

```
./maelstrom test -w g-counter --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 3 --rate 100 --time-limit 20 --nemesis partition
```

### Challenge 5: Kafka-Style Log

---

TODO!

### Challenge 6a: Single-Node, Totally Available Transactions

---

For challenge 6, the `node_runtime` fn should look like this:

```
node_runtime::<kv_store::KVStoreBody, kv_store::KVStoreNode>(node, tx, rx);
```

Challenge directions can be found [here](https://fly.io/dist-sys/6a/).
The following test given in the fly.io directions passes:

```

./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 20 --rate 1000 --concurrency 2n --consistency-models read-uncommitted --availability total

```

### Challenge 6b: Totally-Available, Read Uncommitted Transactions

---

Challenge directions can be found [here](https://fly.io/dist-sys/6b/).
The following two tests given in the fly.io directions passes:

Test1:

```

./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-uncommitted

```

Test2:

```

./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-uncommitted --availability total --nemesis partition

```

Both of the above tests pass, but a note in the fly.io directions
says that "There’s currently an issue in the Maelstrom checker that prohibits detection of G0 anomalies." Hopefully when this bug is fixed I can
run this set of tests again.

### Challenge 6C: Totally-Available, Read Committed Transactions

---

Challenge directions can be found [here](https://fly.io/dist-sys/6b/).
The following test given in the fly.io directions passes:

```

./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-committed --availability total –-nemesis partition

```
