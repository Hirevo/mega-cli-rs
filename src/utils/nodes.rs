/// Constructs the full path to a node, by visiting all of its parents.
pub fn construct_full_path(nodes: &mega::Nodes, node: &mega::Node) -> String {
    let mut full_path = node
        .parent()
        .and_then(|handle| nodes.get_node_by_handle(handle))
        .map(|parent_node| construct_full_path(nodes, parent_node))
        .unwrap_or_default();

    full_path.push('/');
    full_path.push_str(node.name());

    full_path
}

/// Constructs the relative path to a node from a reference (root) node.
pub fn construct_relative_path(
    nodes: &mega::Nodes,
    root: &mega::Node,
    node: &mega::Node,
) -> String {
    if node == root {
        return node.name().to_string();
    }

    let maybe_path = node
        .parent()
        .and_then(|handle| nodes.get_node_by_handle(handle))
        .map(|parent_node| construct_relative_path(nodes, root, parent_node));

    if let Some(mut path) = maybe_path {
        path.push('/');
        path.push_str(node.name());

        path
    } else {
        node.name().to_string()
    }
}
