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