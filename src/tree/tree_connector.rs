use colored::Color;
use url::Url;

use crate::tree::linkable::Linkable;
use crate::tree::printable::Printable;
use crate::tree::tree_node::TreeNode;

pub(crate) struct TreeConnectorNode<T: TreeNode<T>> {
    name: String,
    children: Vec<T>,
}

impl<T: TreeNode<T>> TreeNode<T> for TreeConnectorNode<T> {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_color(&self, _indent: usize) -> Color {
        Color::Red
    }

    fn is_container(&self) -> bool {
        true
    }

    fn get_children(&self) -> &Vec<T>
    where
        Self: Sized,
    {
        &self.children
    }

    fn update_children(&mut self, children: Vec<T>)
    where
        Self: Sized,
    {
        self.children = children;
    }
}

impl<T: TreeNode<T>> Linkable<T> for TreeConnectorNode<T> {
    fn get_url(&self) -> Option<Url> {
        None
    }
}

impl<T: Printable<T>> Printable<T> for TreeConnectorNode<T> {
    fn should_print(&self) -> bool {
        true
    }
}

pub fn connect_trees<T: TreeNode<T>>(tree_list: Vec<T>, name: String) -> TreeConnectorNode<T> {
    let mut result_tree = TreeConnectorNode {
        name,
        children: Vec::new(),
    };
    // let mut result_tree = TreeConnectorNode<T>::new(ScrapeType::Folder, Url::parse("https://ilias.kit.edu").unwrap(), name);

    result_tree.children = tree_list;
    result_tree
}
