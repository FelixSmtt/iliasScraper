use crate::course::Course;
use crate::tree::local_tree_node::LocalTreeNode;
use std::fs;
use std::path::PathBuf;

pub(crate) fn build_tree(course: &Course, base_path: PathBuf) -> LocalTreeNode {
    build_tree_recursive(base_path.join(course.name.clone()))
}

fn build_tree_recursive(current_path: PathBuf) -> LocalTreeNode {
    let is_folder = current_path.is_dir();
    let name = current_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut node = LocalTreeNode::new(is_folder, current_path.to_path_buf(), name.clone());

    if is_folder {
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries.flatten() {
                let child_node = build_tree_recursive(entry.path());
                node.children.push(child_node);
            }
        }
    }

    node
}
