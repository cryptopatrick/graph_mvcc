<h1 align="center">
  <br>
  <a href="https://www.cryptopatrick.com/projects/graph_mvcc">
    <img 
      src="https://github.com/cryptopatrick/factory/blob/master/img/100days/graph_mvcc.png" 
      alt="Title" 
      width="200"
    />
  </a>
  <br>
  GRAPH_MVCC
  <br>
</h1>


<h4 align="center">
  Rust implementation of 
  <a href="https://en.wikipedia.org/wiki/Multiversion_concurrency_control" target="_blank">
    Multiversion Concurrency Control</a> for graph databases.</h4>

<p align="center">
  <a href="https://crates.io/crates/graph_mvcc" target="_blank">
    <img src="https://img.shields.io/crates/v/graph_mvcc" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/graph_mvcc" target="_blank">
    <img src="https://img.shields.io/crates/d/graph_mvcc" alt="Downloads"/>
  </a>
  <a href="https://github.com/cryptopatrick/graph_mvcc/actions" target="_blank">
    <img src="https://img.shields.io/github/actions/workflow/status/cryptopatrick/graph_mvcc/rust.yml?branch=main&label=CI&style=flat-square" alt="Test workflow status"/>
  </a>
  <a href="https://docs.rs/graph_mvcc" target="_blank">
    <img src="https://docs.rs/graph_mvcc/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/crates/l/graph_mvcc?style=flat-square" alt="GitHub license"/>
  </a>
</p>

<p align="center">
  <a href="#what-is-graph-mvcc">What is Graph MVCC</a> •
  <a href="#features">Features</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#license">License</a>
</p>

## 🛎 Important Notices
* This is version **0.2.0** and is still in **active development**
* The current version is **not recommended for production use** ([Work in Progress](#work-in-progress))

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#what-is-graph-mvcc"> What is Graph MVCC</a></li>
    <li><a href="#features"> Features</a></li>
      <ul>
        <li><a href="#core-features"> Core Features</a></li>
        <li><a href="#technical-features">Technical Features</a></li>
      </ul>
    <li><a href="#how-to-use"> How to Use</a></li>
    <li><a href="#work-in-progress"> Work in Progress</a></li>
    <li><a href="#documentation"> Documentation</a></li>
    <li><a href="#author"> Author</a></li>
    <li><a href="#support"> Support</a>
    <li><a href="#contributing"> Contributing</a></li>
    <li><a href="#license">License</a></li>
    </li>
  </ol>
</details>

## 🤔 What is Graph MVCC

`graph_mvcc` is a Rust crate implementing a Multiversion Concurrency Control (MVCC) graph database. This crate provides a transactional graph data structure with support for nodes, edges, and snapshot isolation. It ensures atomic operations, consistent views of the graph state, and conflict detection for concurrent transactions.

### Use Cases

- **Transactional Graph Operations**: Build graph databases with ACID guarantees
- **Concurrent Access**: Handle multiple simultaneous transactions safely  
- **Graph Analytics**: Implement complex graph algorithms with consistent data views
- **Research & Development**: Experiment with MVCC concepts in graph structures
- **Educational Tools**: Learn about transaction isolation and concurrency control


## 📷 Features

###  Core Features
- **Transactional Operations**: Supports atomic transactions with commit and rollback capabilities
- **Snapshot Isolation**: Ensures each transaction sees a consistent view of the graph at the time it starts
- **Node and Edge Management**: Allows creation and manipulation of nodes and edges with type-based collision detection
- **MVCC**: Implements multiversion concurrency control to manage concurrent transactions without conflicts

###  **Technical Features**
- **Deterministic Design**: Single-threaded deterministic implementation
- **UUID-based Identifiers**: Proper Node/Edge Management with unique identifiers for all nodes and edges
- **Graph Traversal**: Support for path-based graph traversal using edge types
- **Temporary Transactions**: Automatic transaction creation for null transaction operations
- **State Management**: Proper tracking of active transactions and rollback actions
- **Collision Detection**: Type-based edge collision detection following MVCC principles

## 🚙 How to Use

### Installation

Install with cargo.

```bash
cargo add graph_mvcc
```

### Example

```rust
use graph_mvcc::Graph;

fn main() {
    let mut graph = Graph::new();
    let mut tx = graph.start_transaction();

    // Add nodes
    let node1 = graph.add_node(&mut tx);
    let node2 = graph.add_node(&mut tx);

    // Add an edge between nodes
    graph.add_edge(&mut tx, &node1, &node2, "CONNECTS".to_string()).unwrap();

    // Commit the transaction
    graph.commit_transaction(&tx).unwrap();
    
    println!("Successfully created graph with 2 nodes and 1 edge");
}
```

## Work in Progress

I've bumped the version to 0.2.0, but there's still a lot of work to do.
This project can be used to experiment with MVCC on a graph data structure, but
please shelve any ideas of using the current version (0.2.0) in production.
My main goal now is to try and offload all node- and edge-handling, to the excellent
[graph](https://crates.io/crates/graph) crate.

## 📚 Documentation

Comprehensive documentation is available at [docs.rs/graph_mvcc](https://docs.rs/graph_mvcc), including:
- API reference for all public types and functions
- Examples of transaction management
- Performance considerations and best practices
- Implementation details of MVCC for graphs

## 🖊 Author

<span>CryptoPatrick  <a href="https://x.com/cryptopatrick"><img width="30" height="30" src="https://github.com/cryptopatrick/factory/blob/master/img/x.png" /></a>  </span>  
Keybase Verification:  
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support
Leave a ⭐ If you think this project is cool.  
If you think it has helped in any way, consider [!buying me a coffee!](https://github.com/cryptopatrick/factory/blob/master/img/bmc-button.png)

## 🤝 Contributing

Found a bug? Missing a specific feature?
Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for details on:
- Code style and testing requirements
- Submitting bug reports and feature requests
- Development setup and workflow

## 🗄 License
This project is licensed under MIT. See [LICENSE](LICENSE) for details.