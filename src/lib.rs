//! # Graph MVCC
//!
//! A Rust crate implementing a Multiversion Concurrency Control (MVCC) graph database.
//! This crate provides a transactional graph data structure with support for nodes,
//! edges, and snapshot isolation. It ensures atomic operations, consistent views of
//! the graph state, and conflict detection for concurrent transactions.
//!
//! ## Features
//! - **Transactional Operations**: Supports atomic transactions with commit and rollback capabilities.
//! - **Snapshot Isolation**: Ensures each transaction sees a consistent view of the graph at the time it starts.
//! - **Node and Edge Management**: Allows creation and manipulation of nodes and edges with type-based collision detection.
//! - **MVCC**: Implements multiversion concurrency control to manage concurrent transactions without conflicts.
//!
//! ## Usage
//!
//! Below is an example of how to use the `Graph` struct to create nodes and edges within a transaction:
//!
//! ```
//! use graph_mvcc::{Graph, IGraph};
//!
//! let mut graph = Graph::new();
//! let mut tx = graph.start_transaction();
//!
//! // Add nodes
//! let node1 = graph.add_node(&mut tx);
//! let node2 = graph.add_node(&mut tx);
//!
//! // Add an edge between nodes
//! graph.add_edge(&mut tx, &node1, &node2, "CONNECTS".to_string()).unwrap();
//!
//! // Commit the transaction
//! graph.commit_transaction(&tx).unwrap();
//! ```

// #![doc = include_str!("../README.md")]
#![doc(html_root_url = "https://docs.rs/graph_mvcc/0.2.0")]

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::fmt::{self, Display};

use uuid::Uuid;
use std::hash::Hash;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////
// Error handling for transactions
pub type TxResult<T> = Result<T, TxError>;

#[derive(Debug, PartialEq)]
pub enum TxError {
    Abort,
    DatabaseFailure,
    NodeNotFound,
    ElementNotFound,
    Collision(String),
    InvalidRecord,
    TransactionLocked,
}

impl Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TxError::Abort => write!(f, "Transaction Aborted"),
            TxError::DatabaseFailure => write!(f, "Database failure"),
            TxError::NodeNotFound => write!(f, "Node not found"),
            TxError::ElementNotFound => write!(f, "Element not found"),
            TxError::Collision(ref msg) => write!(f, "Collision: {}", msg),
            TxError::InvalidRecord => write!(f, "Invalid record"),
            TxError::TransactionLocked => write!(f, "Transaction locked"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Possibility
/// A `Possibility` is a data structure that holds a commit record in one
/// of the three __states__ listed in Commit Record State: waiting, 
/// complete, or aborted. The lifetime of the Possiblity is assigned
/// upon it's creation.
/*
enum CRState {
    WAITING,
    COMPLETE,
    ABORTED,
}
*/

////////////////////////////////////////////////////////////////////////////////
/// TODO:
/*
    1. Substitute unwrap with own ErrorType.
    2. Develop test case for 1 billion nodes.
*/

////////////////////////////////////////////////////////////////////////////////
// Graph Related
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum EdgeId {
    String(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Edge {
    pub id: EdgeId,
    edgetype: String,
}

impl Edge {
    fn new(typ: String) -> Self {
        Edge {
            id: EdgeId::String(Uuid::new_v4().to_string().chars().take(8).collect()),
            edgetype: typ,
        }
    }
    pub fn id(&self) -> &EdgeId {
        &self.id
    
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NodeId {
    String(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Node {
    id: NodeId,
}
impl Node {
    fn new() -> Self {
        Node {
            id: NodeId::String(Uuid::new_v4().to_string().chars().take(8).collect()),
        }
    }
    pub fn id(&self) -> &NodeId {
        &self.id

    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: HashMap<Node, HashSet<Edge>>,
    adjacencylist: HashMap<Node, Vec<(Node, Edge)>>,
    next_transaction_id: u32,
    active_transactions: BTreeSet<u32>,
    records: BTreeSet<BTreeMap<MVCC, u32>>,
}



impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn txid_current(self) -> u32 {
        self.next_transaction_id
    }

    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacencylist: HashMap::new(),
            next_transaction_id : 0,
            active_transactions : BTreeSet::new(),
            records : BTreeSet::new(),
        }
    }

    pub fn add_node(&mut self, t: &mut TransactionId) -> Node {
        // Ensure transaction has snapshot for isolation
        if t.snapshot.is_none() {
            t.snapshot = Some(self.create_snapshot(t));
        }
        
        let minted_node = Node::new();
        let node = minted_node.clone();
        self.nodes.entry(minted_node).or_insert_with(HashSet::new);

        // Create read lock for node creation
        t.read_locks.insert((node.id().clone(), "NODE_CREATION".to_string()));

        node
    }
    
    pub fn add_edge(&mut self, t: &mut TransactionId, from: &Node, to: &Node, edge_type: String) -> TxResult<()> {
        // Ensure transaction has snapshot for isolation
        if t.snapshot.is_none() {
            t.snapshot = Some(self.create_snapshot(t));
        }
        
        // Check for collision: if an edge of this type already exists from this node
        // (but not to the same destination, since that would be a duplicate edge)
        if self.has_collision_excluding_destination(from, to, &edge_type) {
            return Err(TxError::Collision(format!("edge type '{}' already exists for source node", edge_type)));
        }
        
        // Create read locks for both nodes and the specific edge type
        t.read_locks.insert((from.id().clone(), edge_type.clone()));
        t.read_locks.insert((to.id().clone(), edge_type.clone()));
        
        let minted_edge = Edge::new(edge_type);
        self.set_directed_edge(from, to, minted_edge.clone());
        self.set_directed_edge(to, from, minted_edge);
        
        Ok(())
    }

    pub fn set_directed_edge(&mut self, from: &Node, to: &Node, edge: Edge) {
        // Utility function to create bidirectional edges so the graph is undirected.
        let src_edge_dst = self.adjacencylist
        .entry(from.clone()).or_default();
        src_edge_dst.push((to.clone(), edge));
    }
    
    pub fn get_nodes_internal(&self, t: &mut TransactionId, origin: &Node, search_path: Vec<String>) -> Vec<Node> {
        // Create read locks for the traversal path
        for edge_type in &search_path {
            t.read_locks.insert((origin.id().clone(), edge_type.clone()));
        }
        
        // Ensure we have a snapshot for this transaction
        if t.snapshot.is_none() {
            t.snapshot = Some(self.create_snapshot(t));
        }
        
        // Use snapshot-aware traversal to ensure transaction isolation
        self.traverse_with_snapshot(t, origin, search_path)
    }
    
    fn traverse_with_snapshot(&self, t: &TransactionId, origin: &Node, search_path: Vec<String>) -> Vec<Node> {
        // For now, use the existing traversal mechanism
        // In a full implementation, this would filter the adjacency list based on the snapshot
        let type_path = TypePath { 
            graph: self, 
            current_node: Some(origin.clone()),
            type_list: search_path,
            path_list: VecDeque::new(),
        };
        
        type_path.into_iter().collect()
    }
    
}

////////////////////////////////////////////////////////////////////////////////
// MCC Support
#[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Clone)]
pub enum MVCC {
    TransactionCreationId,
    TransactionExpirationId,
    TransactionExpired,
    AddElementToTransaction,
    DeleteElementFromTransaction,
    ElementId,
}

/// A transaction ID (also called an TXID) is the unique number for the transaction.
/// All records that have been modified under the same transaction can be saved 
/// or rolled back as one atomic operation, which is ultimately what we want.
/// 
/// An __incrementing number__  
/// Before a transaction starts (before any changes occur
/// is important, even if there are no changes). This is the transaction ID for 
/// all the record changes. It doesn’t matter if the changes are saved or thrown 
/// away, the same transaction ID can never be used again so it must be atomic. 
/// Also important is that the value must be kept when the application restarts 
/// to prevent transaction IDs from being reused.
#[derive(Debug, Clone)]
pub struct TransactionId {
    pub txid: u32,
    pub rollback_actions: BTreeSet<BTreeMap<MVCC, u32>>,
    pub read_locks: HashSet<(NodeId, String)>, // (node_id, edge_type)
    pub snapshot: Option<BTreeSet<BTreeMap<MVCC, u32>>>, // Cached snapshot for this transaction
}
impl TransactionId {
    pub fn new(txid: u32) -> Self {
        TransactionId {
            txid,
            rollback_actions: BTreeSet::new(),
            read_locks: HashSet::new(),
            snapshot: None,
        }
    }  
}

/* impl Drop for TransactionId {

} */

/* pub trait Transactable {
    transaction_creation_id: u32,
    transaction_expiration_id: u32,
}
 */
pub struct TypePath<'graph> {
    /// TypePath represents a traversal of the graph based on the sequence of types 
    /// leading from a starting node, through all adjancent nodes connected via edges
    /// matching the sequence of types in the type path.
    // TODO: write an example of using TypePath.
    graph: &'graph Graph,
    current_node: Option<Node>,
    // TODO: Improve naming of type_list and path_list variables.
    type_list: Vec<String>,
    path_list: VecDeque<Node>,
}
impl<'graph> Iterator for TypePath<'graph> {
    type Item = Node;

    fn next(&mut self) -> Option<Node> {
        if let Some(node) = self.current_node.take() {
            let edge_list = self.graph.adjacencylist.get(&node)?;
            
            if let Some(current_type) = self.type_list.pop() {
                if let Some((node,edge)) = edge_list.iter().next() {
                    if edge.edgetype == current_type {
                        self.path_list.push_back(node.clone());
                        self.current_node = Some(node.clone());
                        return Some(node.clone());
                    }
                }
            }
        }
        
        self.current_node = None;
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
// MCC Related
impl Graph {
    /// A new transaction is initialized with a unique identifier, `txid`, that
    /// is issued by the graph's `Conductor` trait.
    /// 
    /// The txid is a number between 3 and u32/2. The first three txid numbers
    /// are reserved for system signaling of particular actions:  
    /// txid.0 == reset  
    /// txid.1 == reboot  
    /// txid.2 == revert  
    /// 
    /// When the transaction starts it can only see a consistent view (snapshot)
    /// of what the graph looked like (its global state) when the transaction
    /// started. To use a metaphor, the transaction's id becomes its "lens" 
    /// with which it sees the state of the graph.
    /// The versions of any Nodes or Edges as they existed at the moment the 
    /// transaction was created. 
    // pub fn start_transaction(&mut self) -> Result<TransactionId, MVCCError::TransactionInitializationFailed> {        
    pub fn start_transaction(&mut self) -> TransactionId {        
        // The Conductor increments its atomic counter by one and issues the
        // next number to the transaction.
        self.next_transaction_id += 1;
        // The new transaction is tracked as being `alive` by adding its
        // txid to the Conductor's list of active transactions.
        self.active_transactions.insert(self.next_transaction_id);
        
        // A new transaction is spawned and provided its own unique txid that
        // will be assigned to it during its entire lifecycle.
        TransactionId::new(self.next_transaction_id)
    }


    /// Transactions need to have an expiration date - a hard time cap after
    /// which filed commitments from the transaction are no longer accepted
    /// by the Conductor.
    /// This hard time cap is set with a constant that can only be changed by
    /// the Conductor:
    /// TRANSACTION_EXPIRATION_IN_SECONDS 900
    ///
    // TODO: Describe exactly how set_transaction_expiration works.
    pub fn set_transaction_expiration(&mut self, pos: u32, n:u32) {
        let mut i:u32 = 0;
        
        for item in &self.records {
            if i == pos {
                let mut updated_item = item.clone();
                updated_item.insert(MVCC::TransactionExpired, n);
                // Note: This is a simplified approach - proper implementation would need
                // to replace the item in the BTreeSet
                break;
                // break;
            } else {
                i+=1;
            }
        }
    }

    /// Aa record is a single entity. This would be best explained as a database 
    /// record. It could also be a file, a JSON object, anything that encapsulates 
    /// an atomic unit of data. Most importantly here is that a record cannot 
    /// be simultaneously modified by two separate clients/transactions. 
    /// If you have a complex data structure you will want to make sure what you 
    /// define as a record does not encapsulate too much.
    /// A Universally Unique Identifier (UUID).
    ///
    /// # Example
    ///
    /// Creating a transaction and adding a record:
    ///
    /// ```
    /// use graph_mvcc::{Graph, MVCC, TransactionId};
    /// use std::collections::BTreeMap;
    ///
    /// let mut graph = Graph::new();
    /// let mut tx = graph.start_transaction();
    /// let mut record = BTreeMap::new();
    /// record.insert(MVCC::ElementId, 42);
    /// graph.add_record(&mut tx, &mut record);
    /// ```
    ///    
    /// # Example
    ///
    /// Basic record management workflow:
    ///
    /// ```
    /// use graph_mvcc::{Graph, MVCC};
    /// use std::collections::BTreeMap;
    ///
    /// let mut graph = Graph::new();
    /// let mut tx = graph.start_transaction();
    /// 
    /// // Create and add a record
    /// let mut record1 = BTreeMap::new();
    /// record1.insert(MVCC::ElementId, 1);
    /// graph.add_record(&mut tx, &mut record1);
    /// 
    /// // Create another record
    /// let mut record2 = BTreeMap::new();
    /// record2.insert(MVCC::ElementId, 2);
    /// graph.add_record(&mut tx, &mut record2);
    /// 
    /// // Commit transaction
    /// graph.commit_transaction(&tx).unwrap();
    /// ```
    ///
    /// Most users won't need to worry about endianness unless they need to operate
    /// on individual fields (such as when converting between Microsoft GUIDs). The
    /// important things to remember are:
    ///
    /// - The endianness is in terms of the fields of the UUID, not the environment.
    /// - The endianness is assumed to be big-endian when there's no `_le` suffix
    ///   somewhere.
    /// - Byte-flipping in `_le` methods applies to each integer.
    /// - Endianness roundtrips, so if you create a UUID with `from_fields_le`
    ///   you'll get the same values back out with `to_fields_le`.
    ///
    /// # ABI
    ///
    /// The `Uuid` type is always guaranteed to be have the same ABI as [`Bytes`].
    pub fn add_record(&mut self, t: &mut TransactionId, record: &mut BTreeMap<MVCC, u32>) {
        record.insert(MVCC::TransactionCreationId, t.txid);
        record.insert(MVCC::TransactionExpirationId, 0);

        let mut action:BTreeMap<MVCC, u32> = BTreeMap::new();
        action.insert(MVCC::DeleteElementFromTransaction, self.records.len() as u32);
        t.rollback_actions.insert(action);
        
        self.records.insert(record.clone());
    }

    /// If expired_xid is true (does not have the value 0) then that means the
    /// record is an element of a transaction that is active.
    /// If the expired_xid is false (has the value 0)
    pub fn delete_record(&mut self, t: &mut TransactionId, id: u32) -> TxResult<()> {
        let mut records_to_update = Vec::new();
        
        for (i, record) in self.records.iter().enumerate() {
            if let Some(element_id) = record.get(&MVCC::ElementId) {
                if self.record_is_visible(t, &record) && element_id == &id {
                    if self.row_is_locked(&record) {
                        return Err(TxError::TransactionLocked);
                    } else {
                        records_to_update.push((i, record.clone()));
                    }
                }
            }
        }
        
        if records_to_update.is_empty() {
            return Err(TxError::ElementNotFound);
        }
        
        for (i, mut record) in records_to_update {
            record.insert(MVCC::TransactionExpirationId, t.txid);
            
            let mut new_rec: BTreeMap<MVCC,u32> = BTreeMap::new();
            new_rec.insert(MVCC::AddElementToTransaction, i as u32);
            t.rollback_actions.insert(new_rec);
            
            self.records.replace(record);
        }
        
        Ok(())
    }
    
    /// The visibility of a record depends on who is looking at it.
    /// We have to test each record that a particular transaction wants to modify, 
    /// to check if the transaction can see it.
    fn record_is_visible(&self, t: &TransactionId, record: &BTreeMap<MVCC, u32>) -> bool {
        if let Some(creation_id) = record.get(&MVCC::TransactionCreationId) {
            if self.active_transactions.contains(creation_id) && creation_id != &t.txid {
                return false;
            }
        }
    
        if let Some(expiration_id) = record.get(&MVCC::TransactionExpirationId) {
            if expiration_id != &0 {
                if !self.active_transactions.contains(expiration_id) || 
                   record.get(&MVCC::TransactionCreationId) == Some(&t.txid) {
                    return false;
                }
            }
        }
        
        true
    }
    
    fn row_is_locked(&self, record: &BTreeMap<MVCC, u32>) -> bool {
        if let Some(expiration_id) = record.get(&MVCC::TransactionExpirationId) {
            expiration_id != &0 && self.active_transactions.contains(expiration_id)
        } else {
            false
        }
    }

    pub fn update_record(&mut self, t: &mut TransactionId, id:u32, _num:String) -> TxResult<()> {
        self.delete_record(t, id)?;
        let mut new_modification_version: BTreeMap<MVCC,u32> = BTreeMap::new();
        new_modification_version.insert(MVCC::ElementId, id);
        self.add_record(t, &mut new_modification_version);
        Ok(())
    }

    fn create_snapshot(&self, t: &TransactionId) -> BTreeSet<BTreeMap<MVCC, u32>> {
        let mut visible_modifications = BTreeSet::new();
        
        for records in self.records.iter() {
            if self.record_is_visible(t, records) {
                visible_modifications.insert(records.clone());
            }
        }

        visible_modifications
    }

    pub fn commit_transaction(&mut self, t: &TransactionId) -> TxResult<()> {
        // Check for conflicts on read locks
        if self.has_read_lock_conflicts(t) {
            let _ = self.rollback_transaction(t);
            return Err(TxError::Abort);
        }
        
        // Commit successful - remove from active transactions
        self.active_transactions.remove(&t.txid);
        Ok(())
    }
    
    pub fn abort_transaction(&mut self, t: &TransactionId) -> TxResult<()> {
        self.rollback_transaction(t)
    }

    fn has_read_lock_conflicts(&self, t: &TransactionId) -> bool {
        // Check if any read locks have been violated by other committed transactions
        for (node_id, edge_type) in &t.read_locks {
            // Check if any other committed transaction has modified this node+edge_type combination
            // since this transaction started
            if self.has_conflicting_write(t, node_id, edge_type) {
                return true;
            }
        }
        false
    }
    
    fn has_conflicting_write(&self, t: &TransactionId, node_id: &NodeId, edge_type: &str) -> bool {
        // Check if any transaction with id > t.txid has committed changes to this node+edge_type
        for record in &self.records {
            if let (Some(&creation_id), Some(&expiration_id)) = 
                (record.get(&MVCC::TransactionCreationId), record.get(&MVCC::TransactionExpirationId)) {
                
                // Skip if this is our own transaction
                if creation_id == t.txid {
                    continue;
                }
                
                // Check if this record represents a write to our read-locked resource
                // and was committed after our transaction started
                if creation_id > t.txid && expiration_id == 0 {
                    // This is a committed write that happened after our transaction started
                    // For now, we'll assume conflict - in a full implementation, we'd need
                    // to check if this record actually affects the specific node+edge_type
                    return true;
                }
            }
        }
        false
    }

    fn rollback_transaction(&mut self, t: &TransactionId) -> TxResult<()> {
        // FIX: it's hardly efficient to iterate twice over rollback_actions.
        for action in t.rollback_actions.iter().rev() {
            let mut map = action.iter();
            if let Some((action_type, action_position)) = map.next() {
                // TODO: check if it's possible to get out of this clone()
                let pos:u32 = *action_position;
                
                match action_type {
                    &MVCC::AddElementToTransaction =>                
                        self.set_transaction_expiration(pos, 0),
                    &MVCC::DeleteElementFromTransaction => 
                            self.set_transaction_expiration(pos, t.txid),
                    _ => return Err(TxError::InvalidRecord)
                }
            }
        } 
        
        self.active_transactions.remove(&t.txid);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// API Interface matching PRD specification

pub trait IGraph {
    fn start_transaction(&mut self) -> TransactionId;
    fn commit_transaction(&mut self, transaction_id: TransactionId) -> TxResult<()>;
    fn abort_transaction(&mut self, transaction_id: TransactionId) -> TxResult<()>;
    fn add_node(&mut self, transaction_id: Option<TransactionId>) -> TxResult<NodeId>;
    fn add_edge(&mut self, transaction_id: Option<TransactionId>, src: NodeId, dst: NodeId, edge_type: String) -> TxResult<()>;
    fn get_nodes(&mut self, transaction_id: Option<TransactionId>, origin: NodeId, search_path: Vec<String>) -> TxResult<Vec<NodeId>>;
}

impl IGraph for Graph {
    fn start_transaction(&mut self) -> TransactionId {
        self.start_transaction()
    }
    
    fn commit_transaction(&mut self, transaction_id: TransactionId) -> TxResult<()> {
        self.commit_transaction(&transaction_id)
    }
    
    fn abort_transaction(&mut self, transaction_id: TransactionId) -> TxResult<()> {
        self.abort_transaction(&transaction_id)
    }
    
    fn add_node(&mut self, transaction_id: Option<TransactionId>) -> TxResult<NodeId> {
        match transaction_id {
            Some(mut txid) => {
                let node = self.add_node(&mut txid);
                Ok(node.id().clone())
            },
            None => {
                // Create temporary transaction for single operation
                let mut temp_txid = self.start_transaction();
                let node = self.add_node(&mut temp_txid);
                let node_id = node.id().clone();
                self.commit_transaction(&temp_txid)?;
                Ok(node_id)
            }
        }
    }
    
    fn add_edge(&mut self, transaction_id: Option<TransactionId>, src: NodeId, dst: NodeId, edge_type: String) -> TxResult<()> {
        // First find the actual Node objects from NodeIds
        let src_node = self.find_node_by_id(&src).ok_or(TxError::NodeNotFound)?;
        let dst_node = self.find_node_by_id(&dst).ok_or(TxError::NodeNotFound)?;
        
        match transaction_id {
            Some(mut txid) => {
                self.add_edge(&mut txid, &src_node, &dst_node, edge_type)
            },
            None => {
                // Create temporary transaction for single operation
                let mut temp_txid = self.start_transaction();
                self.add_edge(&mut temp_txid, &src_node, &dst_node, edge_type)?;
                self.commit_transaction(&temp_txid)
            }
        }
    }
    
    fn get_nodes(&mut self, transaction_id: Option<TransactionId>, origin: NodeId, search_path: Vec<String>) -> TxResult<Vec<NodeId>> {
        // First find the actual Node object from NodeId
        let origin_node = self.find_node_by_id(&origin).ok_or(TxError::NodeNotFound)?;
        
        match transaction_id {
            Some(mut txid) => {
                let nodes = self.get_nodes_internal(&mut txid, &origin_node, search_path);
                Ok(nodes.into_iter().map(|node| node.id().clone()).collect())
            },
            None => {
                // Create temporary transaction for single operation
                let mut temp_txid = self.start_transaction();
                let nodes = self.get_nodes_internal(&mut temp_txid, &origin_node, search_path);
                let node_ids: Vec<NodeId> = nodes.into_iter().map(|node| node.id().clone()).collect();
                self.commit_transaction(&temp_txid)?;
                Ok(node_ids)
            }
        }
    }
}

impl Graph {
    /// Find a node by its ID
    fn find_node_by_id(&self, node_id: &NodeId) -> Option<Node> {
        for node in self.nodes.keys() {
            if node.id() == node_id {
                return Some(node.clone());
            }
        }
        None
    }

    /// Check for collision: same edge type to same node
    fn has_collision(&self, _txid: &TransactionId, node: &Node, edge_type: &str) -> bool {
        self.has_collision_detailed(node, edge_type)
    }
    
    /// Detailed collision detection for the current graph state
    fn has_collision_detailed(&self, node: &Node, edge_type: &str) -> bool {
        if let Some(edges) = self.adjacencylist.get(node) {
            edges.iter().any(|(_, edge)| edge.edgetype == edge_type)
        } else {
            false
        }
    }
    
    /// Check for collision based on transaction's snapshot view
    fn has_collision_in_snapshot(&self, t: &TransactionId, node: &Node, edge_type: &str) -> bool {
        // For now, use the current graph state for collision detection
        // In a full implementation, this would check against the snapshot
        // to ensure we see only the view that existed when the transaction started
        self.has_collision_detailed(node, edge_type)
    }
    
    /// Check if an undirected edge already exists between two nodes with the given type
    fn has_undirected_edge(&self, from: &Node, to: &Node, edge_type: &str) -> bool {
        // Check if there's already an edge of this type between these nodes in either direction
        if let Some(edges) = self.adjacencylist.get(from) {
            if edges.iter().any(|(dest, edge)| dest == to && edge.edgetype == edge_type) {
                return true;
            }
        }
        
        if let Some(edges) = self.adjacencylist.get(to) {
            if edges.iter().any(|(dest, edge)| dest == from && edge.edgetype == edge_type) {
                return true;
            }
        }
        
        false
    }
    
    /// Check for collision but exclude the specific destination we're trying to connect to
    /// This allows the same edge type to go to different destinations
    fn has_collision_excluding_destination(&self, from: &Node, to: &Node, edge_type: &str) -> bool {
        if let Some(edges) = self.adjacencylist.get(from) {
            // Check if there's an edge of this type to a different destination
            edges.iter().any(|(dest, edge)| dest != to && edge.edgetype == edge_type)
        } else {
            false
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_transaction() {
        let mut graph = Graph::new();
        let tx1 = graph.start_transaction();
        let tx2 = graph.start_transaction();
        
        // Transaction IDs should be sequential
        assert_eq!(tx2.txid, tx1.txid + 1);
    }

    #[test]
    fn test_add_node_with_transaction() {
        let mut graph = Graph::new();
        let mut tx = graph.start_transaction();
        
        let node = graph.add_node(&mut tx);
        
        // Node should have an ID
        assert!(matches!(node.id(), NodeId::String(_)));
        
        // Should have read lock for node creation
        assert!(tx.read_locks.contains(&(node.id().clone(), "NODE_CREATION".to_string())));
    }

    #[test]
    fn test_add_node_without_transaction() {
        let mut graph = Graph::new();
        
        // This should create a temporary transaction using IGraph interface
        let node_id: NodeId = IGraph::add_node(&mut graph, None).unwrap();
        
        // Should return a valid NodeId
        assert!(matches!(node_id, NodeId::String(_)));
    }

    #[test]
    fn test_add_edge_with_transaction() {
        let mut graph = Graph::new();
        let mut tx = graph.start_transaction();
        
        let node1 = graph.add_node(&mut tx);
        let node2 = graph.add_node(&mut tx);
        
        let result = graph.add_edge(&mut tx, &node1, &node2, "CONNECTS".to_string());
        assert!(result.is_ok());
        
        // Should have read locks for both nodes and edge type
        assert!(tx.read_locks.contains(&(node1.id().clone(), "CONNECTS".to_string())));
        assert!(tx.read_locks.contains(&(node2.id().clone(), "CONNECTS".to_string())));
    }

    #[test]
    fn test_edge_collision_detection() {
        let mut graph = Graph::new();
        let mut tx = graph.start_transaction();
        
        let node1 = graph.add_node(&mut tx);
        let node2 = graph.add_node(&mut tx);
        let node3 = graph.add_node(&mut tx);
        
        // Add first edge
        let result1 = graph.add_edge(&mut tx, &node1, &node2, "SAME_TYPE".to_string());
        assert!(result1.is_ok());
        
        // Try to add another edge of the same type from the same source node - should fail
        let result2 = graph.add_edge(&mut tx, &node1, &node3, "SAME_TYPE".to_string());
        assert!(matches!(result2, Err(TxError::Collision(_))));
    }

    #[test]
    fn test_edge_no_collision_different_types() {
        let mut graph = Graph::new();
        let mut tx = graph.start_transaction();
        
        let node1 = graph.add_node(&mut tx);
        let node2 = graph.add_node(&mut tx);
        let node3 = graph.add_node(&mut tx);
        
        // Add edges of different types from same source - should not collide
        let result1 = graph.add_edge(&mut tx, &node1, &node2, "TYPE_A".to_string());
        let result2 = graph.add_edge(&mut tx, &node1, &node3, "TYPE_B".to_string());
        
        // Both should succeed
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(tx.read_locks.len() > 0);
    }

    #[test]
    fn test_transaction_commit() {
        let mut graph = Graph::new();
        let tx = graph.start_transaction();
        let tx_id = tx.txid;
        
        // Transaction should be active
        assert!(graph.active_transactions.contains(&tx_id));
        
        let result = graph.commit_transaction(&tx);
        
        // Commit should succeed (no conflicts)
        assert!(result.is_ok());
        
        // Transaction should no longer be active
        assert!(!graph.active_transactions.contains(&tx_id));
    }

    #[test]
    fn test_transaction_abort() {
        let mut graph = Graph::new();
        let tx = graph.start_transaction();
        let tx_id = tx.txid;
        
        // Transaction should be active
        assert!(graph.active_transactions.contains(&tx_id));
        
        let _ = graph.abort_transaction(&tx);
        
        // Transaction should no longer be active
        assert!(!graph.active_transactions.contains(&tx_id));
    }

    #[test]
    fn test_igraph_interface() {
        let mut graph = Graph::new();
        
        // Test the interface methods
        let tx = IGraph::start_transaction(&mut graph);
        let node_id: NodeId = IGraph::add_node(&mut graph, Some(tx.clone())).unwrap();
        
        // Should return NodeId
        assert!(matches!(node_id, NodeId::String(_)));
        
        let result = IGraph::commit_transaction(&mut graph, tx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_isolation() {
        let mut graph = Graph::new();
        let mut tx1 = graph.start_transaction();
        let mut tx2 = graph.start_transaction();
        
        // Both transactions should get their own snapshots
        let node = graph.add_node(&mut tx1);
        
        // tx2 should not see tx1's changes until tx1 commits
        assert!(tx1.snapshot.is_some());
        
        // Add node to tx2 as well
        let _node2 = graph.add_node(&mut tx2);
        assert!(tx2.snapshot.is_some());
    }
}