use std::fmt::Debug;
use crate::traits::trait_listable::Listable;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
const PADDING: usize = 2;

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct SortedListState<T: Clone + Debug + Eq + Ord + Listable> {
    unique: bool,
    selected: usize,
    scroll_anchor: usize,
    entries: Vec<T>,
}

impl<'a, T: Clone + Debug + Eq + Ord + Listable> SortedListState<T> {
    pub fn new(unique: bool) -> Self {
        SortedListState{
            unique,
            scroll_anchor: 0,
            selected: 0,
            entries: vec![],
        }
    }

    // get state ----------------------------------------------------

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_entry(&'a self) -> Option<&'a T> {
        self.entries.get(self.selected)
    }

    pub fn entries(&self) -> &Vec<T> {
        &self.entries
    }

    // -- selection -------------------------------------------------

    pub fn select_next(&mut self, offset: usize) {
        let initial_next_selected = (self.selected + offset).min(self.entries.len().saturating_sub(1));

        let res = self.entries.iter()
            .enumerate()
            .skip(initial_next_selected)
            .find(|(_i, e)| e.is_selectable());
        match res {
            Some((i, _e)) => self.selected = i,
            None => self.select_end(),
        }
    }
    pub fn select_prev(&mut self, offset: usize) {
        let initial_next_selected = self.selected.saturating_sub(offset);

        let res = self.entries.iter()
            .enumerate()
            .take(initial_next_selected + 1)
            .rev()
            .find(|(_i, e)| e.is_selectable());
        match res {
            Some((i, _e)) => self.selected = i,
            None => self.select_start(),
        }
    }
    pub fn select_start(&mut self) {
        let res = self.entries.iter()
            .enumerate()
            .find(|(_i, e)| e.is_selectable());
        match res {
            Some((i, _e)) => self.selected = i,
            None => self.selected = 0,
        }
    }
    pub fn select_end(&mut self) {
        let res = self.entries.iter()
            .enumerate()
            .rev()
            .find(|(_i, e)| e.is_selectable());
        match res {
            Some((i, _e)) => self.selected = i,
            None => self.selected = 0,
        }
    }

    // -- insert ----------------------------------------------------

    pub fn add(&mut self, element: T) {
        if let Some(index) = match self.entries.binary_search(&element) {
            Ok (index) => if self.unique {None} else {Some(index)},
            Err(index) => Some(index),
        } {
            let first_selectable = self.entries.iter()
                .enumerate()
                .find_map(|(i,e)| if e.is_selectable() {Some(i)} else {None})
                .unwrap_or(usize::MAX);
            self.entries.insert(index, element);
            if first_selectable >= index {
                self.select_start();
            } else {
                if index <= self.selected {
                    self.scroll_anchor += 1;
                    self.selected += 1;
                }
            }
        }
    }

    pub fn replace_all(&mut self, elements: Vec<T>) {
        self.entries = elements;
        self.select_start();
        self.scroll_anchor = 0;
    }

    // -- render ----------------------------------------------------

    /// will only output correct data if update_scroll
    pub fn view(&mut self, term_height: usize) -> (Vec<T>, usize) {
        self.update_scroll(term_height);

        let viewable_elements = self.entries.iter()
            .skip(self.scroll_anchor)
            .take(term_height)
            .cloned()
            .collect::<Vec<T>>();
        let selected = self.selected.saturating_sub(self.scroll_anchor);

        (
            viewable_elements,
            selected,
        )
    }

    fn update_scroll(&mut self, render_height: usize) {
        // min/max possible
        let list_bot = self.entries.len().saturating_sub(1);
        let anchor_max = list_bot.saturating_sub(render_height.saturating_sub(1));

        // min/max visible
        let window_top = self.scroll_anchor;
        let window_bot = self.scroll_anchor + render_height.saturating_sub(1);

        // min/max selected thresholds
        let selected_top = window_top + PADDING;
        let selected_bot = window_bot.saturating_sub(PADDING);

        // min/max new positions for scroll_anchor when selection hits top/bot thresholds
        let new_top = (self.selected).saturating_sub(PADDING);
        let new_bot = (self.selected + PADDING + 1).saturating_sub(render_height);

        // dbg!(self.selected);
        // dbg!(self.scroll_anchor);
        // dbg!(list_bot);
        // dbg!(anchor_max);
        // dbg!(window_top);
        // dbg!(window_bot);
        // dbg!(selected_top);
        // dbg!(selected_bot);
        // dbg!(new_top);
        // dbg!(new_bot);

        if self.selected < selected_top {self.scroll_anchor = new_top                }
        if self.selected > selected_bot {self.scroll_anchor = new_bot.min(anchor_max)}
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Clone, Copy)]
    #[derive(Debug)]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct DummyElement{
        selectable: bool,
    }
    impl DummyElement {
        pub fn new(selectable: bool) -> DummyElement {
            DummyElement{
                selectable,
            }
        }
    }
    impl Listable for DummyElement {
        fn to_list_item<'a>(self, _width: usize) -> tui::widgets::ListItem<'a> {
            todo!()
        }
        fn is_selectable(&self) -> bool {
            self.selectable
        }
    }

    #[test]
    fn test_sorted_list_scroll_down() {
        let term_height = 10;
        let mut state = SortedListState{
            unique       : false,
            selected     : 0,
            scroll_anchor: 0,
            entries      : vec![DummyElement::new(true); 30],
        };

        for i in 0..29 {
            let i = i+1;
            println!("selected: {}", i);

            state.select_next(1);
            state.update_scroll(term_height);

            assert_eq!(state, SortedListState{
                unique       : false,
                selected     :  i,
                scroll_anchor: (i+1).saturating_sub(term_height-PADDING).min(30-term_height),
                entries      : vec![DummyElement::new(true); 30],
            });
        }
    }

    #[test]
    fn test_sorted_list_scroll_up() {
        let term_height = 10;
        let mut state = SortedListState{
            unique       : false,
            selected     : 29,
            scroll_anchor: 29 - term_height,
            entries      : vec![DummyElement::new(true); 30],
        };

        for i in 0..29 {
            let i = 29 - 1 - i;
            println!("selected: {}", i);

            state.select_prev(1);
            state.update_scroll(term_height);

            assert_eq!(state, SortedListState{
                unique       : false,
                selected     : i,
                scroll_anchor: i.saturating_sub(PADDING).min(30-term_height),
                entries      : vec![DummyElement::new(true); 30],
            });
        }
    }

    #[test]
    fn test_sorted_list_scroll_past_selectable() {
        let mut state = SortedListState{
            unique       : false,
            selected     : 0,
            scroll_anchor: 0,
            entries      : vec![
                DummyElement::new(true ),
                DummyElement::new(false),
                DummyElement::new(true ),
            ],
        };

        state.select_next(1);
        assert_eq!(state, SortedListState{
            unique       : false,
            selected     : 2,
            scroll_anchor: 0,
            entries      : vec![
                DummyElement::new(true ),
                DummyElement::new(false),
                DummyElement::new(true ),
            ],
        });

        state.select_prev(1);
        assert_eq!(state, SortedListState{
            unique       : false,
            selected     : 0,
            scroll_anchor: 0,
            entries      : vec![
                DummyElement::new(true ),
                DummyElement::new(false),
                DummyElement::new(true ),
            ],
        });
    }

    #[test]
    fn test_sorted_list_add_element() {
        let mut state: SortedListState<DummyElement> = SortedListState{
            unique       : false,
            selected     : 0,
            scroll_anchor: 0,
            entries      : vec![],
        };

        state.add(DummyElement::new(false));
        state.add(DummyElement::new(true ));
        state.add(DummyElement::new(true ));
        state.add(DummyElement::new(true ));


        assert_eq!(state, SortedListState{
            unique       : false,
            selected     : 1,
            scroll_anchor: 0,
            entries      : vec![
                DummyElement::new(false),
                DummyElement::new(true ),
                DummyElement::new(true ),
                DummyElement::new(true ),
            ],
        });


    }
}

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
