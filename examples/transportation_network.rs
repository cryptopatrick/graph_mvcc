use graph_mvcc::{Graph, IGraph, NodeId, TxError};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum NodeType {
    City,
    Airport,
    TrainStation,
}

#[derive(Debug, Clone)]
enum EdgeType {
    Road,
    Rail, 
    AirToLondon,      // Specific air routes to comply with MVCC constraints
    AirToTokyo,
    AirToParis,
    RailToParis,
    RailToTokyo,
    RoadToParis,
    RoadToTokyo,
}

impl EdgeType {
    fn as_string(&self) -> String {
        match self {
            EdgeType::Road => "Road".to_string(),
            EdgeType::Rail => "Rail".to_string(),
            EdgeType::AirToLondon => "AirToLondon".to_string(),
            EdgeType::AirToTokyo => "AirToTokyo".to_string(),
            EdgeType::AirToParis => "AirToParis".to_string(),
            EdgeType::RailToParis => "RailToParis".to_string(),
            EdgeType::RailToTokyo => "RailToTokyo".to_string(),
            EdgeType::RoadToParis => "RoadToParis".to_string(),
            EdgeType::RoadToTokyo => "RoadToTokyo".to_string(),
        }
    }
}

fn main() -> Result<(), TxError> {
    println!("🚀 MVCC Transportation Network Example");
    println!("======================================");
    
    // Create the graph
    let mut graph = Graph::new();
    let mut node_names = HashMap::new(); // Track node names for display
    
    // Start a transaction for building the network
    let tx1 = graph.start_transaction();
    println!("Started transaction {}", tx1.txid);
    
    // Create nodes using the IGraph interface
    println!("\n📍 Creating Transportation Nodes...");
    
    let nodes = create_transportation_nodes(&mut graph, &mut node_names)?;
    
    // Create connections between nodes
    println!("\n🛣️ Creating Transportation Connections...");
    create_connections(&mut graph, &nodes, &node_names)?;
    
    // Test collision detection
    println!("\n⚠️ Testing Collision Detection...");
    test_collision_detection(&mut graph, &nodes)?;
    
    // Test concurrent transactions with read locks
    println!("\n🔄 Testing Concurrent Transactions & Read Locks...");
    test_concurrent_transactions(&mut graph, &nodes)?;
    
    // Test graph traversal
    println!("\n🗺️ Testing Graph Traversal...");
    test_graph_traversal(&mut graph, &nodes, &node_names)?;
    
    println!("\n🎉 Transportation Network Example Complete!");
    
    Ok(())
}

fn create_transportation_nodes(graph: &mut Graph, node_names: &mut HashMap<NodeId, String>) -> Result<Vec<NodeId>, TxError> {
    let mut nodes = Vec::new();
    
    // Create nodes with meaningful names
    let locations = vec![
        ("New York City", NodeType::City),
        ("London", NodeType::City),
        ("Tokyo", NodeType::City),
        ("Paris", NodeType::City),
        ("JFK Airport", NodeType::Airport),
        ("Heathrow Airport", NodeType::Airport), 
        ("Narita Airport", NodeType::Airport),
        ("Charles de Gaulle Airport", NodeType::Airport),
        ("Penn Station NYC", NodeType::TrainStation),
        ("St Pancras London", NodeType::TrainStation),
        ("Tokyo Station", NodeType::TrainStation),
        ("Gare du Nord Paris", NodeType::TrainStation),
    ];
    
    // Create all nodes using IGraph interface
    for (name, node_type) in locations {
        let node_id = IGraph::add_node(graph, None)?;
        node_names.insert(node_id.clone(), name.to_string());
        println!("  ➕ Created {:?}: {} (ID: {:?})", node_type, name, node_id);
        nodes.push(node_id);
    }
    
    Ok(nodes)
}

fn create_connections(graph: &mut Graph, nodes: &[NodeId], node_names: &HashMap<NodeId, String>) -> Result<(), TxError> {
    // Create connections that comply with MVCC collision constraints
    if nodes.len() >= 12 {
        // Air connections - each with unique edge types
        println!("  ✈️ Creating Air Routes...");
        IGraph::add_edge(graph, None, nodes[4].clone(), nodes[5].clone(), EdgeType::AirToLondon.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[4]), 
                get_node_name(node_names, &nodes[5]),
                EdgeType::AirToLondon.as_string());
        
        IGraph::add_edge(graph, None, nodes[4].clone(), nodes[6].clone(), EdgeType::AirToTokyo.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[4]), 
                get_node_name(node_names, &nodes[6]),
                EdgeType::AirToTokyo.as_string());
        
        IGraph::add_edge(graph, None, nodes[5].clone(), nodes[7].clone(), EdgeType::AirToParis.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[5]), 
                get_node_name(node_names, &nodes[7]),
                EdgeType::AirToParis.as_string());
        
        // Rail connections
        println!("  🚂 Creating Rail Routes...");
        IGraph::add_edge(graph, None, nodes[8].clone(), nodes[9].clone(), EdgeType::RailToParis.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[8]), 
                get_node_name(node_names, &nodes[9]),
                EdgeType::RailToParis.as_string());
        
        IGraph::add_edge(graph, None, nodes[10].clone(), nodes[11].clone(), EdgeType::RailToTokyo.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[10]), 
                get_node_name(node_names, &nodes[11]),
                EdgeType::RailToTokyo.as_string());
        
        // Road connections between cities
        println!("  🛣️ Creating Road Routes...");
        IGraph::add_edge(graph, None, nodes[0].clone(), nodes[1].clone(), EdgeType::RoadToParis.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[0]), 
                get_node_name(node_names, &nodes[1]),
                EdgeType::RoadToParis.as_string());
        
        IGraph::add_edge(graph, None, nodes[1].clone(), nodes[2].clone(), EdgeType::RoadToTokyo.as_string())?;
        println!("    {} ↔ {} ({})", 
                get_node_name(node_names, &nodes[1]), 
                get_node_name(node_names, &nodes[2]),
                EdgeType::RoadToTokyo.as_string());
    }
    
    Ok(())
}

fn test_collision_detection(graph: &mut Graph, nodes: &[NodeId]) -> Result<(), TxError> {
    if nodes.len() >= 12 {
        // Try to add a duplicate specific air connection (should fail because it's exactly the same)
        println!("  🔍 Attempting to add duplicate AirToLondon connection...");
        match IGraph::add_edge(graph, None, nodes[4].clone(), nodes[5].clone(), EdgeType::AirToLondon.as_string()) {
            Err(TxError::Collision(msg)) => {
                println!("  ✅ Duplicate connection correctly rejected: {}", msg);
            },
            Ok(_) => {
                println!("  ❌ Duplicate connection should have been rejected!");
            },
            Err(e) => {
                println!("  ❓ Unexpected error: {:?}", e);
            }
        }
        
        // Try to add a new air connection from same node (should fail due to collision constraint)
        println!("  🔍 Attempting to add second air connection from same node (should fail)...");
        match IGraph::add_edge(graph, None, nodes[4].clone(), nodes[7].clone(), EdgeType::AirToParis.as_string()) {
            Err(TxError::Collision(msg)) => {
                println!("  ✅ Collision correctly detected: {}", msg);
            },
            Ok(_) => {
                println!("  ❌ This should have triggered a collision (node already has AirToTokyo)!");
            },
            Err(e) => {
                println!("  ❓ Unexpected error: {:?}", e);
            }
        }
        
        // Try to add a completely different edge type (should succeed)
        println!("  🔍 Attempting to add different edge type (should succeed)...");
        match IGraph::add_edge(graph, None, nodes[0].clone(), nodes[3].clone(), EdgeType::RoadToParis.as_string()) {
            Ok(_) => {
                println!("  ✅ Different edge type added successfully (no collision)");
            },
            Err(e) => {
                println!("  ❌ Different edge type should have succeeded: {:?}", e);
            }
        }
    }
    
    Ok(())
}

fn test_concurrent_transactions(graph: &mut Graph, nodes: &[NodeId]) -> Result<(), TxError> {
    if nodes.len() >= 4 {
        // Start two concurrent transactions
        let tx1 = IGraph::start_transaction(graph);
        let tx2 = IGraph::start_transaction(graph);
        
        println!("  Started concurrent transactions: {} and {}", tx1.txid, tx2.txid);
        
        // Both transactions read from the same nodes
        println!("  🔍 Testing concurrent read operations...");
        
        // Transaction 1: Try to traverse using AirToLondon
        match IGraph::get_nodes(graph, Some(tx1.clone()), nodes[4].clone(), vec![EdgeType::AirToLondon.as_string()]) {
            Ok(destinations) => {
                println!("    Tx{}: Found {} AirToLondon destinations", tx1.txid, destinations.len());
            },
            Err(e) => {
                println!("    Tx{}: Error during traversal: {:?}", tx1.txid, e);
            }
        }
        
        // Transaction 2: Try to traverse using RoadToParis
        match IGraph::get_nodes(graph, Some(tx2.clone()), nodes[0].clone(), vec![EdgeType::RoadToParis.as_string()]) {
            Ok(destinations) => {
                println!("    Tx{}: Found {} RoadToParis destinations", tx2.txid, destinations.len());
            },
            Err(e) => {
                println!("    Tx{}: Error during traversal: {:?}", tx2.txid, e);
            }
        }
        
        // Commit both transactions
        match IGraph::commit_transaction(graph, tx1.clone()) {
            Ok(_) => println!("  ✅ Transaction {} committed successfully", tx1.txid),
            Err(e) => println!("  ❌ Transaction {} failed to commit: {:?}", tx1.txid, e),
        }
        
        match IGraph::commit_transaction(graph, tx2.clone()) {
            Ok(_) => println!("  ✅ Transaction {} committed successfully", tx2.txid),
            Err(e) => println!("  ❌ Transaction {} failed to commit: {:?}", tx2.txid, e),
        }
    }
    
    Ok(())
}

fn test_graph_traversal(graph: &mut Graph, nodes: &[NodeId], node_names: &HashMap<NodeId, String>) -> Result<(), TxError> {
    if nodes.len() >= 12 {
        println!("  🗺️ Testing single-hop traversals...");
        
        // Test AirToLondon connections from JFK (node[4])
        let air_destinations = IGraph::get_nodes(graph, None, nodes[4].clone(), vec![EdgeType::AirToLondon.as_string()])?;
        println!("    ✈️ From {}: {} AirToLondon destinations found", 
                get_node_name(node_names, &nodes[4]), air_destinations.len());
        
        // Test AirToTokyo connections from JFK (node[4])  
        let air_tokyo = IGraph::get_nodes(graph, None, nodes[4].clone(), vec![EdgeType::AirToTokyo.as_string()])?;
        println!("    ✈️ From {}: {} AirToTokyo destinations found", 
                get_node_name(node_names, &nodes[4]), air_tokyo.len());
        
        // Test RoadToParis connections from NYC (node[0])
        let road_destinations = IGraph::get_nodes(graph, None, nodes[0].clone(), vec![EdgeType::RoadToParis.as_string()])?;
        println!("    🛣️ From {}: {} RoadToParis destinations found", 
                get_node_name(node_names, &nodes[0]), road_destinations.len());
        
        // Test RailToParis connections from Penn Station (node[8])
        let rail_destinations = IGraph::get_nodes(graph, None, nodes[8].clone(), vec![EdgeType::RailToParis.as_string()])?;
        println!("    🚂 From {}: {} RailToParis destinations found", 
                get_node_name(node_names, &nodes[8]), rail_destinations.len());
        
        println!("  🔄 Testing multi-hop traversals...");
        
        // Multi-hop: Try AirToLondon then AirToParis
        let multi_hop = IGraph::get_nodes(graph, None, nodes[4].clone(), 
                                        vec![EdgeType::AirToLondon.as_string(), EdgeType::AirToParis.as_string()])?;
        println!("    ✈️✈️ AirToLondon→AirToParis from {}: {} final destinations", 
                get_node_name(node_names, &nodes[4]), multi_hop.len());
        
        // Test with temporary transaction (None parameter)
        println!("  ⚡ Testing temporary transactions...");
        let temp_destinations = IGraph::get_nodes(graph, None, nodes[1].clone(), vec![EdgeType::RoadToTokyo.as_string()])?;
        println!("    🔄 Temporary transaction traversal: {} destinations found", temp_destinations.len());
    }
    
    Ok(())
}

// Helper function to get node name for display
fn get_node_name(node_names: &HashMap<NodeId, String>, node_id: &NodeId) -> String {
    node_names.get(node_id)
             .map(|s| s.clone())
             .unwrap_or_else(|| format!("{:?}", node_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transportation_network_example() {
        // Basic test that the example can run
        assert!(main().is_ok());
    }
}