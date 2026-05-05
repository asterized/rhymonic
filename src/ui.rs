mod components;
mod ellipsize;
mod helpers;
mod interface;
mod update;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Songs,
    Queue,
    Albums,
    Album(usize),
}
