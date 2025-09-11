Graph MVCC

[GitHub Actions workflow status]: https://img.shields.io/github/actions/workflow/status/cryptopatrick/graph_mvcc/rust.yml?branch=main&label=CI&style=flat-square
[actions]: https://github.com/cryptopatrick/graph_mvcc/actions/workflows/rust.yml?query=branch%3Amain
[Latest version on crates.io]: https://img.shields.io/crates/v/graph_mvcc?style=flat-square
[crates.io]: https://crates.io/crates/graph_mvcc/
[License: MIT]: https://img.shields.io/crates/l/graph_mvcc?style=flat-square
[license]: https://choosealicense.com/licenses/mit/

---

# Graph MVCC
A Rust crate implementing a Multiversion Concurrency Control (MVCC) graph database.
This crate provides a transactional graph data structure with support for nodes,
edges, and snapshot isolation. It ensures atomic operations, consistent views of
the graph state, and conflict detection for concurrent transactions.


## Crate Features
- **Transactional Operations**: Supports atomic transactions with commit and rollback capabilities.
- **Snapshot Isolation**: Ensures each transaction sees a consistent view of the graph at the time it starts.
- **Node and Edge Management**: Allows creation and manipulation of nodes and edges with type-based collision detection.
- **MVCC**: Implements multiversion concurrency control to manage concurrent transactions without conflicts.

### Technical Features
  - Thread-safe Design: single-threaded deterministic implementation
  - Proper Node/Edge Management: UUID-based unique identifiers for all nodes and edges
  - Graph Traversal: Support for path-based graph traversal using edge types
  - Temporary Transactions: Automatic transaction creation for null transaction operations
  - Robust State Management: Proper tracking of active transactions and rollback actions

## Usage
Below is an example of how to use the `Graph` struct to create nodes and edges within a transaction:

```rust
use graph_mvcc::{Graph, IGraph};

let mut graph = Graph::new();
let mut tx = graph.start_transaction();


// Add nodes
let node1 = graph.add_node(&mut tx);
let node2 = graph.add_node(&mut tx);

// Add an edge between nodes
graph.add_edge(&mut tx, &node1, &node2, "CONNECTS".to_string()).unwrap();

// Commit the transaction
graph.commit_transaction(&tx).unwrap();
```

## Work in Progress
I've bumped the version to 0.2.0, but there's still a lot of work to do.
This project can be used to experiment with MVCC on a graph data structure, but
please shelve any ideas of using the current version (0.2.0) in production.
My main goal now is to try and offload all node- and edge-handling, to the excellent
[graph](https://crates.io/crates/graph) crate.