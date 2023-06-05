use crate::traits::trait_listable::Listable;
use std::fmt::Debug;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::List;
use tui::widgets::ListItem;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct SortedListState<T: Clone + Debug + Eq + Ord + Listable> {
    selected: usize,
    scroll_anchor: usize,
    entries: Vec<T>,
}

impl<'a, T: Clone + Debug + Eq + Ord + Listable> SortedListState<T> {
    pub fn new() -> Self {
        SortedListState{
            scroll_anchor: 0,
            selected: 0,
            entries: vec![],
        }
    }

    // get state ----------------------------------------------------

    pub fn selected_index(&self) -> Option<usize> {
        self.entries.get(self.selected)
            .and_then(|e| match e.is_selectable() {
                true => Some(self.selected),
                false => None,
            })
    }
    pub fn selected_entry(&'a self) -> Option<&'a T> {
        self.entries.get(self.selected)
        .and_then(|e| match e.is_selectable() {
            true => Some(e),
            false => None,
        })
    }
    pub fn selectable(&'a self) -> (Option<usize>, &'a Vec<T>) {
        (
            self.selected_index(),
            &self.entries,
        )
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
        if let Err(index) = self.entries.binary_search(&element) {
            self.entries.insert(index, element);
            if index <= self.selected {
                self.scroll_anchor += 1;
                self.selected += 1;
            }

            if self.selected_entry().is_none() {
                self.select_start()
            }
        }
    }

    pub fn replace_all(&mut self, elements: impl Iterator<Item = T>) {
        self.entries = vec![];
        for element in elements {
            if let Err(index) = self.entries.binary_search(&element) {
                self.entries.insert(index, element);
            }
        }
        if self.selected >= self.entries.len() {
            self.select_end()
        }
        if self.selected_entry().is_none() {
            self.select_next(1);
        }
    }

    // -- render ----------------------------------------------------

    /// `update_scroll` should be run before `render`
    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, is_active: bool) {
        // todo: do style globally somewhere else
        let selected_style = match is_active {
            true => Style::default()
                .fg(Color::White)
                .bg(Color::Red),
            false => Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray),
        };

        let elements = self.entries.iter()
            .map(|element| element.to_list_item(area.width as usize))
            .enumerate()
            .map(|(i, element)| match i == self.selected {
                true => element.style(selected_style),
                false => element,
            })
            .skip(self.scroll_anchor)
            .take(area.height as usize)
            .collect::<Vec<ListItem>>();

        frame.render_widget(List::new(elements), area);
    }

    pub fn update_scroll(&mut self, render_height: usize) {
        let padding = 2;

        // calculate bounds
        let max_anchor_dept = self.entries.len().saturating_sub(render_height);
        let window_top = self.scroll_anchor.saturating_add(padding);
        let window_bot = self.scroll_anchor.saturating_add(render_height.saturating_sub(padding+1));

        // adjust view
        if self.selected < window_top {self.scroll_anchor = self.selected.saturating_sub(padding)}
        if self.selected > window_bot {self.scroll_anchor = self.selected.saturating_add(padding+1).saturating_sub(render_height)}

        // avoid over scroll
        self.scroll_anchor = self.scroll_anchor.min(max_anchor_dept);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
