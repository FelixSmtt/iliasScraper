use crate::tree::tree_node::TreeNode;
use url::Url;

fn link(uri: &str, label: &String) -> String {
    let parameters = "";

    // OSC 8 ; params ; URI ST <name> OSC 8 ;; ST
    format!("\x1B]8;{};{}\x1B\\{}\x1B]8;;\x1B\\", parameters, uri, label)
}

pub trait Linkable<T: TreeNode<T>>: TreeNode<T> {
    fn get_url(&self) -> Option<Url>;

    fn represent(&self) -> String {
        if let Some(url) = self.get_url() {
            link(url.as_str(), self.get_name())
        } else {
            self.get_name().clone()
        }
    }
}
