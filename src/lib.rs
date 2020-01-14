#[doc(hidden)]
pub use ::proc_macro::with_concat_ident
    as __with_concat_ident__
;

// #[cfg(FALSE)]
#[macro_export]
macro_rules! with_concat_ident {(
    $($tt:tt)*
) => (
    $crate::as_item! { 
        $crate::__with_concat_ident__! {
            $($tt)*
        }
    }
)}

#[macro_export]
#[doc(hidden)]
macro_rules! as_item { ($it:item) => ($it) }
