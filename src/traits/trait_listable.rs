use tui::widgets::ListItem;

pub trait Listable {
    fn to_list_item(&self, width: usize) -> ListItem;
    fn is_selectable(&self) -> bool;
}
