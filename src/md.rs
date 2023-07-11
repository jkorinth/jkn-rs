#[macro_export]
macro_rules! md {
    ($($args:expr),*) => {{
        ::termimad::MadSkin::default().print_text(&format!($($args),*).as_str());
    }}
}

pub use md;
