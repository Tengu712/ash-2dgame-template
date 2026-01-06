use proc_macro::*;

/// `[||]`で囲まれた識別子とリテラルを連結して1個の識別子にするマクロ
///
/// ```rs
/// cat_ids! {
///     // fn foo_bar12() {}
///     fn [|foo _bar 12|]() {}
/// }
/// ```
///
/// WARN: グループおよび記号は無視される。
#[proc_macro]
pub fn cat_ids(input: TokenStream) -> TokenStream {
    fn is_surrounded_by_pipe(input: TokenStream) -> bool {
        let mut iter = input.into_iter();
        if !matches!(iter.next(), Some(TokenTree::Punct(p)) if p.as_char() == '|') {
            return false;
        }
        if !matches!(iter.last(), Some(TokenTree::Punct(p)) if p.as_char() == '|') {
            return false;
        }
        true
    }

    fn process_tokens(input: TokenStream) -> TokenStream {
        let mut trees = Vec::new();
        for tt in input.into_iter() {
            match tt {
                TokenTree::Group(g)
                    if matches!(g.delimiter(), Delimiter::Bracket)
                        && is_surrounded_by_pipe(g.stream()) =>
                {
                    let id = g
                        .stream()
                        .into_iter()
                        .filter_map(|tt| match tt {
                            TokenTree::Ident(n) => Some(n.to_string()),
                            TokenTree::Literal(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .collect::<String>();
                    trees.push(TokenTree::Ident(Ident::new(&id, Span::call_site())));
                }
                TokenTree::Group(g) => {
                    trees.push(TokenTree::Group(Group::new(
                        g.delimiter(),
                        process_tokens(g.stream()),
                    )));
                }
                _ => trees.push(tt),
            }
        }
        trees.into_iter().collect()
    }

    process_tokens(input)
}
