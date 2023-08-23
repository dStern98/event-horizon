# This is an attempt to pass the Maelstrom-Jepsen Distributed Systems Challenges.

### 1

```
./maelstrom test -w echo --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 10
```

### 2

```
 ./maelstrom test -w unique-ids --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```

### 6a

```
./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 1 --time-limit 20 --rate 1000 --concurrency 2n --consistency-models read-uncommitted --availability total

```

### 6b

```
./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-uncommitted
```

```
./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-uncommitted --availability total --nemesis partition
```

### 6C

```
 ./maelstrom test -w txn-rw-register --bin /mnt/c/Users/dstern/Documents/Dev/Practice_Code/Github_Projects/event-horizon/target/debug/event-horizon --node-count 2 --concurrency 2n --time-limit 20 --rate 1000 --consistency-models read-committed --availability total –-nemesis partition
```
