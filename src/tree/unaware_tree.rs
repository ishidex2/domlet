/*!
 *
 *      Tree that is unaware of it's surrounding or it's parents,
 *      Since rusts borrow checker doesn't allow for that,
 *      it will be replaced in the future with a more versatile implementation
 *
 */

use super::BasicTree;

pub struct UnawareTree<N> {
    /**
     * The value that the tree will represent
     */
    pub node: N,

    /**
     * Children nodes
     */
    pub children: Option<Vec<UnawareTree<N>>>,
}

impl<N> UnawareTree<N> {}

/**
 * Generic tree construction
 */
impl<N> BasicTree<N> for UnawareTree<N> {
    fn new_node(node: N) -> Self {
        Self {
            node,
            children: Some(vec![]),
        }
    }

    fn child_count(&self) -> usize {
        match self.children {
            Some(children) => children.len(),
            None => 0
        }
    }

    fn new_leaf(leaf: N) -> Self {
        Self {
            node: leaf,
            children: None,
        }
    }

    fn add_child(&mut self, node: UnawareTree<N>) {
        if let Some(children) = &mut self.children {
            children.push(node)
        }
    }

    fn get_child(&self, n: usize) -> Option<&Self> {
        self.children.as_ref()?.get(n)
    }

    fn get_child_mut(&mut self, n: usize) -> Option<&mut Self> {
        self.children.as_mut()?.get_mut(n)
    }

    fn get_node(&self) -> &N {
        &self.node
    }

    fn get_node_mut(&mut self) -> &mut N {
        &mut self.node
    }
}
