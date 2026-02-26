use crate::tree::printable::Printable;
use crate::tree::tree_node::TreeNode;

#[allow(dead_code)]
pub trait ComparableTreeNode<T: Printable<T>>: TreeNode<T> {
    fn compare_as_remote<O: TreeNode<O>>(&self, other: &O) -> Self;
    fn compare_as_local<O: TreeNode<O> + Clone>(&self, other: &O) -> O;
}

pub(crate) fn compare_trees<T: TreeNode<T> + Clone, K: TreeNode<K>>(
    local_tree: &K,
    remote_tree: &T,
) -> T {
    let mut result_tree = remote_tree.clone();
    // let mut result_tree = ScrapeObject::new(remote_tree.item_type.clone(), remote_tree.url.clone(), remote_tree.name.clone());

    let mut children: Vec<T> = Vec::new();

    for remote_child in remote_tree.get_children() {
        let mut found = false;

        for local_child in local_tree.get_children() {
            if local_child.get_name() == remote_child.get_name() {
                //println!("Comparing\n: {:?}", local_child);
                //println!("And\n: {:?}", remote_child);

                if local_child.is_container() && remote_child.is_container() {
                    let folder_node = compare_trees(local_child, remote_child);
                    if !folder_node.get_children().is_empty() {
                        children.push(folder_node);
                    }
                }

                found = true;
            }
        }
        if !found {
            children.push(remote_child.clone());
        }
    }

    result_tree.update_children(children);
    result_tree
}
