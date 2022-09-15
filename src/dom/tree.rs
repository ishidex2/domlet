/*!
 *
 *      DOM tree representation
 *
 */

use crate::tree::BasicTree;

use super::node::{Element, ElementAttributes, Node};

pub struct NodeTree<Tree: BasicTree<Node>> {
    pub(super) tree: Tree,
}

impl<Tree: BasicTree<Node>> NodeTree<Tree> {
    /**
     * Constructs `NodeTree` from `Vec<Tree>`
     * Will panic if:
     *  + No tree
     *  + More than 1 tree
     */
    pub fn from_tree_vec(mut trees: Vec<Tree>) -> Self {
        if trees.len() == 0 {
            panic!("DOM tree does not contain any elements");
        }

        if trees.len() > 1 {
            panic!("DOM tree contains more than one root element");
        }

        /*
         * This drain iterator is required to gain ownership of the tree
         */
        Self {
            tree: trees.drain(..).next().unwrap(),
        }
    }
}
