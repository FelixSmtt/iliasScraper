pub trait TreeNode<C: TreeNode<C>> {
    fn get_name(&self) -> &String;
    fn get_color(&self, indent: usize) -> colored::Color;
    fn is_container(&self) -> bool;
    fn get_children(&self) -> &Vec<C>
    where
        Self: Sized;
    fn update_children(&mut self, children: Vec<C>)
    where
        Self: Sized;
}
