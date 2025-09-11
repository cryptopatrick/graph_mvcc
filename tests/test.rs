#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_01() {
        let g = Graph::new();
        let t = g.start_transaction();
        let n1 = g.add_node(t);
        let n2 = g.add_node(t);
        g.add_edge(t, n1, n2, "red");

        assert_eq!(g.get_nodes(Some(t), n1, "red"), vec![n2]);
        assert_eq!(g.get_nodes(Some(t), n1, "blue"), vec![]);
        assert_eq!(g.get_nodes(Some(t), n2, "red"), vec![n1]);
        assert_eq!(g.get_nodes(Some(t), n2, "blue"), vec![]);
    }

    #[test]
    fn test_case_02() {
        let g = Graph::new();
        let t = g.start_transaction();
        let n1 = g.add_node(t);
        let n2 = g.add_node(t);
        let n3 = g.add_node(t);
        g.commit_transaction(t);

        let t1 = g.start_transaction();
        g.add_edge(t1, n1, n2, "red");

        let t2 = g.start_transaction();
        g.add_edge(t2, n1, n3, "blue");

        // Transaction t1 sees the red edges
        assert_eq!(g.get_nodes(Some(t1), n1, "red"), vec![n2]);
        assert_eq!(g.get_nodes(Some(t1), n1, "blue"), vec![]);

        // Transaction t2 sees the blue edges
        assert_eq!(g.get_nodes(Some(t2), n1, "red"), vec![]);
        assert_eq!(g.get_nodes(Some(t2), n1, "blue"), vec![n3]);

        g.commit_transaction(t1);

        // New transactions see the committed red edge, but not the uncommitted blue
        assert_eq!(g.get_nodes(None, n1, "red"), vec![n2]);
        assert_eq!(g.get_nodes(None, n1, "blue"), vec![]);

        // Transaction t2 still sees the same as before
        assert_eq!(g.get_nodes(Some(t2), n1, "red"), vec![]);
        assert_eq!(g.get_nodes(Some(t2), n1, "blue"), vec![n3]);

        // Commit should not fail
        g.commit_transaction(t2);

        // New transactions see all changes
        assert_eq!(g.get_nodes(None, n1, "red"), vec![n2]);
        assert_eq!(g.get_nodes(None, n1, "blue"), vec![n3]);
    }

    #[test]
    #[should_panic]
    fn test_case_03() {
        let g = Graph::new();
        let t = g.start_transaction();
        let n1 = g.add_node(t);
        let n2 = g.add_node(t);
        let n3 = g.add_node(t);
        g.commit_transaction(t);

        let t1 = g.start_transaction();
        g.add_edge(t1, n1, n2, "red");

        let t2 = g.start_transaction();
        g.add_edge(t2, n1, n3, "red");

        // First wins
        g.commit_transaction(t1);

        // Should panic, simulating the exception for conflicting red edges
        g.commit_transaction(t2);
    }

    #[test]
    fn test_case_04() {
        let g = Graph::new();
        let t = g.start_transaction();
        let n1 = g.add_node(t);
        let n2 = g.add_node(t);
        let n3 = g.add_node(t);
        g.add_edge(t, n1, n2, "red");
        g.commit_transaction(t);

        let t1 = g.start_transaction();
        g.add_edge(t1, n2, n3, "blue");

        // Uncommitted transactions should see previously committed nodes and edges
        assert_eq!(g.get_nodes_multi(Some(t1), n1, &["red", "blue"]), vec![n2, n3]);
    }
}