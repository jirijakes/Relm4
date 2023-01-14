use syn::parse::ParseStream;
use syn::token::And;
use syn::{token, Token};

use crate::widgets::parse::if_branch::args_from_index;
use crate::widgets::{parse_util, MatchArm, ParseError, Widget};

impl MatchArm {
    pub(super) fn parse(input: ParseStream<'_>, index: usize) -> Result<Self, ParseError> {
        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
        }

        let pattern = input.parse()?;
        let guard = if input.peek(token::FatArrow) {
            None
        } else {
            Some((input.parse()?, input.parse()?))
        };

        let arrow = input.parse()?;

        let braced;
        let inner_tokens = if input.peek(token::Brace) {
            braced = parse_util::braces(input)?;
            &braced
        } else {
            input
        };

        let attributes = inner_tokens.parse().ok();
        let args = args_from_index(index, input.span());

        let ref_span = input.span();
        let mut widget = Widget::parse(inner_tokens, attributes, Some(args))?;
        widget.ref_token = Some(And { spans: [ref_span] });

        Ok(Self {
            pattern,
            guard,
            arrow,
            widget,
        })
    }
}
