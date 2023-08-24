## About

I have been attempting to pass the Maelstrom distributed system challenges from [fly.io](https://fly.io/dist-sys/). These tests use Maelstrom, a
"workbench for learning distributed systems by writing your own", the Github of which can be found [here](https://github.com/jepsen-io/maelstrom).
I wrote the code in Rust, using Serde & serde_json. Other than that, only the Rust standard library was used. In order to test a binary with Maelstrom, follow the directions for installing Maelstrom found in either
link above. Once a binary is built, maelstrom is invoked with the path to the binary, as well as additional args to configure the test.

### Challenge #1: Echo

---

For challenge 1, `fn main()` in `main.rs` should look like this:

```
let node_metadata = init::MaelstromInit::init_node();
let (tx, rx) = channel::<MaelstromMessage<echo::EchoBody>>();
node_runtime(node_metadata, tx, rx);

```

Challenge directions can be found [here](https://fly.io/dist-sys/1/).
The following test given in the fly.io directions passes:

```
./maelstrom test -w echo --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 10
```

### Challenge #2: Unique ID Generation

---

For challenge 2, `fn main()` in `main.rs` should look like this:

```
let node_metadata = init::MaelstromInit::init_node();
let (tx, rx) = channel::<MaelstromMessage<generate_id::GenerateBody>>();
node_runtime(node_metadata, tx, rx);

```

Challenge directions can be found [here](https://fly.io/dist-sys/2/).
The following test given in the fly.io directions passes:

```
 ./maelstrom test -w unique-ids --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```

### Challenge 5: Kafka-Style Log

---

TODO!

### Challenge 6a: Single-Node, Totally Available Transactions

---

For challenge 6, `fn main()` in `main.rs` should look like this:

```
fn main() {
    let node_metadata = init::MaelstromInit::init_node();
    let (tx, rx) = channel::<MaelstromMessage<kv_store::KVStoreBody>>();
    node_runtime(node_metadata, tx, rx);
}
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
