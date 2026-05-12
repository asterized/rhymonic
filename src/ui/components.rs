mod album_display;
mod album_heading;
mod control_bar;
mod sidebar;
mod song_display;

pub use album_display::album_display;
pub use album_heading::album_heading;
pub use control_bar::control_bar;
pub use sidebar::sidebar;
pub use song_display::display_songs;

macro_rules! load_inline {
    ($name:ident) => {
        LazyLock::new(|| {
            svg::Handle::from_memory(include_bytes!(concat!(
                "resources/",
                stringify!($name),
                ".svg"
            )))
        })
    };
}

macro_rules! load_icon {
    ($name:ident) => {
        #[allow(non_upper_case_globals)]
        static $name: LazyLock<svg::Handle> = super::load_inline!($name);
    };
}

pub(super) use {load_icon, load_inline};
