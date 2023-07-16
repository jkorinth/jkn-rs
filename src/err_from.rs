#[macro_export]
macro_rules! err_from {
    ($ErrorT:ty, $SourceT:ty, $Constr:expr) => {
        impl From<$SourceT> for $ErrorT {
            fn from(err: $SourceT) -> Self {
                $Constr(err)
            }
        }
    };
}

pub use err_from;
