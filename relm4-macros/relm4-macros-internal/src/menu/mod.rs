use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, Ident, LitStr, Path};

mod gen;
mod parse;

#[derive(Debug)]
pub struct Menus {
    items: Punctuated<Menu, Comma>,
}

#[derive(Debug)]
struct Menu {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}

#[derive(Debug)]
enum MenuItem {
    Entry(Box<MenuEntry>),
    Custom(LitStr),
    Section(MenuSection),
}

#[derive(Debug)]
struct MenuEntry {
    string: Expr,
    action_ty: Path,
    value: Option<Expr>,
}

#[derive(Debug)]
struct MenuSection {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}
