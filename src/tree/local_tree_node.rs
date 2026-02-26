use colored::Color;
use std::path::PathBuf;
use url::Url;

use crate::tree::linkable::Linkable;
use crate::tree::printable::Printable;
use crate::tree::tree_comparer::{compare_trees, ComparableTreeNode};
use crate::tree::tree_node::TreeNode;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct LocalTreeNode {
    pub(crate) is_folder: bool,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) children: Vec<LocalTreeNode>,
}

#[allow(dead_code)]
impl LocalTreeNode {
    pub(crate) fn new(is_folder: bool, path: PathBuf, name: String) -> LocalTreeNode {
        if is_folder || path.extension().is_none() {
            let extension = "".to_string();

            return LocalTreeNode {
                is_folder,
                path,
                name,
                extension,
                children: Vec::new(),
            };
        }
        let extension = path.extension().unwrap().to_str().unwrap().to_string();
        let name = name.replace(&format!(".{}", extension), "");

        LocalTreeNode {
            is_folder,
            path,
            name,
            extension,
            children: Vec::new(),
        }
    }
}

impl TreeNode<LocalTreeNode> for LocalTreeNode {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_color(&self, indent: usize) -> Color {
        if indent == 0 {
            Color::Red
        } else if self.is_folder {
            Color::Blue
        } else {
            Color::Green
        }
    }

    fn is_container(&self) -> bool {
        self.is_folder
    }

    fn get_children(&self) -> &Vec<Self> {
        self.children.as_ref()
    }
    fn update_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }
}

impl ComparableTreeNode<LocalTreeNode> for LocalTreeNode {
    fn compare_as_remote<T: TreeNode<T>>(&self, other: &T) -> Self {
        compare_trees(other, self)
    }
    fn compare_as_local<T: TreeNode<T> + Clone>(&self, other: &T) -> T {
        compare_trees(self, other)
    }
}

impl Linkable<LocalTreeNode> for LocalTreeNode {
    fn get_url(&self) -> Option<Url> {
        Url::from_file_path(self.path.as_path()).ok()
    }
}

impl Printable<LocalTreeNode> for LocalTreeNode {
    fn should_print(&self) -> bool {
        true
    }
}
