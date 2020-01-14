#[macro_use]
extern crate with_concat_ident;

fn main ()
{
    macro_rules! foo {(
        $value:expr
        =>
        $($tt:tt)+ // tokens to concat into an ident
    ) => (
        // Define a helper macro that will be called with the generated ident
        macro_rules! helper { ($name:ident) => (
            const $name: i32 = $value;
        )}
        // call the `with_concat_ident!` macro
        with_concat_ident! {
            concat!($($tt)*) => helper!
        }
    )}
    foo! {
        42 => answer _ to _ the _ universe _ the _ life _ and _ everything
    }
    dbg!(answer_to_the_universe_the_life_and_everything);
}
