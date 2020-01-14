extern crate proc_macro;

use ::proc_macro::*;
use ::std::*;

trait IntoTokenStream<Arg> {
    fn into_token_stream (self: Self, arg: Arg) -> TokenStream
    ;
}

struct Hack<T> (marker::PhantomData<T>);

impl<T> Hack<T> {
    fn from (_: &'_ T) -> Self
    {
        Self(Default::default())
    }
}

impl<T : Into<TokenStream>> IntoTokenStream<T> for Hack<T>
{
    fn into_token_stream (self: Self, arg: T) -> TokenStream
    {
        arg.into()
    }
}

impl<T : Into<TokenTree>> IntoTokenStream<T> for &'_ Hack<T>
{
    fn into_token_stream (self: Self, arg: T) -> TokenStream
    {
        let tt: TokenTree = arg.into();
        tt.into()
    }
}

macro_rules! basic_quote_spanned {
    (
        @span $span:tt
        @ret $ret:tt
        @parsing
            # $ident:ident
            $($rest:tt)*
    ) => ({
        let mut ret = $ret;
        ret.extend(iter::once(
            Hack::from(&$ident).into_token_stream($ident)
        ));
        basic_quote_spanned! {
            @span $span
            @ret ret
            @parsing $($rest)*
        }
    });

    (
        @span $span:tt
        @ret $ret:tt
        @parsing
            ( $($group:tt)* )
            $($rest:tt)*
    ) => ({
        let mut ret = $ret;
        ret.extend(iter::once(
            TokenStream::from(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                basic_quote_spanned! { $span => $($group)* },
            )))
        ));
        basic_quote_spanned! {
            @span $span
            @ret ret
            @parsing $($rest)*
        }
    });

    (
        @span $span:tt
        @ret $ret:tt
        @parsing
            { $($group:tt)* }
            $($rest:tt)*
    ) => ({
        let mut ret = $ret;
        ret.extend(iter::once(
            TokenStream::from(TokenTree::Group(Group::new(
                Delimiter::Brace,
                basic_quote_spanned! {  $span => $($group)* },
            )))
        ));
        basic_quote_spanned! {
            @span $span
            @ret ret
            @parsing $($rest)*
        }
    });

    (
        @span $span:tt
        @ret $ret:tt
        @parsing
            [ $($group:tt)* ]
            $($rest:tt)*
    ) => ({
        let mut ret = $ret;
        ret.extend(iter::once(
            TokenStream::from(TokenTree::Group(Group::new(
                Delimiter::Bracket,
                basic_quote_spanned!( $span => $($group)* ),
            )))
        ));
        basic_quote_spanned! {
            @span $span
            @ret ret
            @parsing $($rest)*
        }
    });

    (
        @span $span:tt
        @ret $ret:tt
        @parsing
            $tt:tt
            $($rest:tt)*
    ) => ({
        let mut ret = $ret;
        ret.extend(
            stringify!($tt)
                .parse::<TokenStream>()
                .unwrap()
                .into_iter()
                .map(|mut tt| {
                    tt.set_span($span);
                    tt
                })
        );
        basic_quote_spanned! {
            @span $span
            @ret ret
            @parsing $($rest)*
        }
    });

    (
        @span $span:tt
        @ret $ret:tt
        @parsing
    ) => (
        $ret
    );

    (
        $span:expr =>
        $($input:tt)*
    ) => (
        basic_quote_spanned! {
            @span $span
            @ret { TokenStream::new() }
            @parsing $($input)*
        }
    )
}

macro_rules! basic_quote {(
    $($input:tt)*
) => (basic_quote_spanned! {
    Span::call_site() => $($input)*
})}

fn error (span: Span, err_msg: &'_ str) -> TokenStream
{
    let mut args: TokenTree = Group::new(
        Delimiter::Brace,
        TokenTree::from(Literal::string(err_msg)).into(),
    ).into();
    args.set_span(span);
    basic_quote_spanned! { span =>
        compile_error! #args
    }
}

#[proc_macro] pub
fn with_concat_ident (input: TokenStream) -> TokenStream
{
    let mut last_span = Span::call_site();
    let mut tokens = input.into_iter().inspect(|tt| last_span = tt.span());
    // `concat`
    match tokens.next() {
        | Some(TokenTree::Ident(ident))
            if ident.to_string() == "concat"
        => {},
        | _ => return error(last_span, "Expected `concat"),
    }
    // `!`
    match tokens.next() {
        | Some(TokenTree::Punct(punct))
            if punct.as_char() == '!'
        => {},
        | _ => return error(last_span, "Expected `!`"),
    }
    // args @ `( ... )` / `[ ... ]` / `{ ... }
    let args = match tokens.next() {
        | Some(TokenTree::Group(group))
            if group.delimiter() != Delimiter::None
        => group.stream().into_iter(),
        | _ => return error(last_span, "Expected `(`, `{` or `[`"),
    };
    
    // `=>`
    match (tokens.next(), tokens.next()) {
        | (
            Some(TokenTree::Punct(left)),
            Some(TokenTree::Punct(right)),
        ) if true
            && left.as_char() == '='
            && right.as_char() == '>'
            && left.spacing() == Spacing::Joint
        => {},
        | (tt, _) => return error(
            tt.map(|it| it.span()).unwrap_or(last_span),
            "Expected `=>`",
        ),
    };

    let cb: TokenStream = tokens.collect();

    let mut first = true;
    let concat_result =
        args
        .map(|it: TokenTree| Ok({
            let first = mem::replace(&mut first, false);
            match &it {
                | &TokenTree::Ident(ref ident) => ident.to_string(),
                | &TokenTree::Literal(ref literal)
                    if !first
                => {
                    let ret = literal.to_string();
                    if ret.chars().all(|c| c.is_numeric()) {
                        ret
                    } else {
                        return Err(it);
                    }
                },
                | _ => return Err(it),
            }
        }))
        .collect::<Result<String, TokenTree>>()
    ;
    let ref string_concat: String = match concat_result {
        | Ok(it) => it,
        | Err(tt) => return error(tt.span(), "This token leads to an ill-formed ident"),
    };
    let concat: TokenTree = Ident::new(string_concat, Span::call_site()).into();
    basic_quote! {
        #cb { #concat }
    }
}
