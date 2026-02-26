use colored::*;

use crate::tree::linkable::Linkable;
use crate::tree::tree_node::TreeNode;

pub trait Printable<T: Printable<T>>: TreeNode<T> + Linkable<T> {
    fn should_print(&self) -> bool;

    fn print(&self)
    where
        Self: Sized,
    {
        self.print_node_with_children(0);
    }

    fn print_node_with_children(&self, indent: usize)
    where
        Self: Sized,
    {
        self.print_node(indent);

        let children: &Vec<T> = self.get_children();
        for child in children {
            child.print_node_with_children(indent + 1);
        }
    }

    fn print_node(&self, indent: usize) {
        if !self.should_print() {
            return;
        }

        let prefix: String;
        if indent > 0 {
            let indent_str = "│   ".repeat(indent - 1);
            prefix = format!("{}├── ", indent_str);
        } else {
            prefix = String::from("");
        }

        let color = self.get_color(indent);

        let out = self.represent();
        let colored_name = out.color(color);

        println!("{}{}", prefix, colored_name);
    }
}
