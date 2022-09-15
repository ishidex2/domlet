/*!
 *
 *      Root for everything related to tree data structure
 *
 */

pub mod unaware_tree;

/**
 * Interachangable tree interface
 */
pub trait BasicTree<N>: Sized {
    /**
     * Non-terminating node that will contain children
     */
    fn new_node(node: N) -> Self;

    fn with_children(&mut self, mut children: Vec<Self>) {
        for child in children.drain(..) {
            self.add_child(child);
        }
    }

    fn child_count(&self) -> usize;

    /**
     * Terminating leaf that will not contain any children
     */
    fn new_leaf(leaf: N) -> Self;
    fn add_child(&mut self, child: Self);

    fn get_child(&self, n: usize) -> Option<&Self>;
    fn get_child_mut(&mut self, n: usize) -> Option<&mut Self>;

    fn get_node(&self) -> &N;
    fn get_node_mut(&mut self) -> &mut N;

    fn get_child_node(&self, n: usize) -> Option<&N> {
        self.get_child(n).map(|e| e.get_node())
    }

    fn get_child_node_mut(&mut self, n: usize) -> Option<&mut N> {
        self.get_child_mut(n).map(|e| e.get_node_mut())
    }
}
