use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::ast::*;
use rustc_ast::tokenstream::{TokenStream, TokenTree};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::declare_lint_pass;
use rustc_span::Span;
use rustc_ast::token::TokenKind;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```no_run
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```no_run
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.89.0"]
    pub AVOID_SPECIALIZED_AND_GENERIC_PATTERNS_IN_DECLARATIVE_MACRO,
    pedantic,
    "default lint description"
}
declare_lint_pass!(AvoidSpecializedAndGenericPatternsInDeclarativeMacro => [AVOID_SPECIALIZED_AND_GENERIC_PATTERNS_IN_DECLARATIVE_MACRO]);

impl EarlyLintPass for AvoidSpecializedAndGenericPatternsInDeclarativeMacro {
    fn check_mac_def(&mut self, cx: &EarlyContext<'_>, mac_def: &MacroDef) {
        use rustc_ast::token::Delimiter;
        if !mac_def.macro_rules {
            return;
        }

        let rules = split_into_rules(&mac_def.body.tokens.clone());

        // TODO: (PLeVasseur) - Need to get the proper span of the specialized
        // MacroMatcher; currently on the "fat arrow" (=>)
        for (matcher_ts, _rhs_ts, first_span) in rules {
            // a `macro_rules!` matcher always starts with one `()` / `[]` / `{}` group
            if let Some(TokenTree::Delimited(_, _, Delimiter::Parenthesis, inner)) =
                matcher_ts.iter().next()
            {
                // `inner` is the inside of the parens - now get every slot:
                for slot in collect_all_slots(&inner) {
                    if contains_dollar_ty(&slot.tokens) {
                        eprintln!("Found ");
                    }

                    span_lint_and_help(
                        cx,
                        AVOID_SPECIALIZED_AND_GENERIC_PATTERNS_IN_DECLARATIVE_MACRO,
                        slot.span,
                        format!("avoid specialized matchers in declarative macros alongside generic ones: path {:?}; tokens: {:?}", slot.path, slot.tokens),
                        None,
                        "consider implementing specialized patterns outside of macro",
                    );
                }
            }
        }
    }
}

/// Split a `macro_rules!` body into `(matcher, rhs, first_span)` triples.
fn split_into_rules(body: &TokenStream)
    -> Vec<(TokenStream, TokenStream, Span)>
{
    let mut rules = Vec::new();

    let mut lhs: Vec<TokenTree> = Vec::new();
    let mut rhs: Vec<TokenTree> = Vec::new();
    let mut in_rhs              = false;
    let mut span_start: Option<Span> = None;

    for tt in body.iter() {
        match &tt {
            // 1??  Nested groups: treat as one opaque token
            // TODO: (PLeVasseur) - treat this appropriately by "going into" the Delimited?
            TokenTree::Delimited(..) => {
                if in_rhs { rhs.push(tt.clone()) } else { lhs.push(tt.clone()) }
                continue;
            }
            // 2??  Top-level `=>` / `;`
            TokenTree::Token(tok, _) if !in_rhs && tok.kind == TokenKind::FatArrow => {
                in_rhs = true;
                span_start.get_or_insert(tt.span());
                continue;
            }
            TokenTree::Token(tok, _) if in_rhs && tok.kind == TokenKind::Semi => {
                rules.push((
                    TokenStream::from_iter(lhs.clone()),
                    TokenStream::from_iter(rhs.clone()),
                    span_start.unwrap_or(tt.span()),
                ));
                lhs.clear();
                rhs.clear();
                in_rhs     = false;
                span_start = None;
                continue;
            }
            _ => {}
        }

        // 3??  Ordinary token push to current side
        span_start.get_or_insert(tt.span());
        if in_rhs { rhs.push(tt.clone()) } else { lhs.push(tt.clone()) }
    }

    rules
}

/// One discovered slot.
#[derive(Debug)]
pub struct Slot {
    /// Path from the outermost matcher down to this slot.
    /// `[1, 0]` means: *second* slot at level 0, then *first* slot inside that.
    pub path: Vec<usize>,
    /// The tokens that form the slot.
    pub tokens: TokenStream,
    /// Span of the slot's first token - handy for diagnostics.
    pub span: Span,
}

/// Collect all slots that can be reached by recursively descending into
/// every `( ... )`, `[ ... ]`, and `{ ... } group.
pub fn collect_all_slots(matcher_stream: &TokenStream) -> Vec<Slot> {
    let mut slots = Vec::<Slot>::new();
    collect_level(matcher_stream, &mut Vec::<usize>::new(), &mut slots);
    slots
}

/// Recursive helper - splits `stream` on *top-level* commas, then recurses into
/// every nested group it encounters.
/// TODO: (PLeVasseur) - So it's most common to use `,` or `;` as separator, but
/// it's possible to use any token
/// "The separator token can be any token other than a delimiter or one of the
/// repetition operators, but ; and , are the most common. For instance,
/// $( $i:ident ),* represents any number of identifiers separated by commas.
/// Nested repetitions are permitted."
///  - https://doc.rust-lang.org/nightly/reference/macros-by-example.html#r-macro.decl.repetition.separator
/// delimeters are {}, [], ()
///  - https://doc.rust-lang.org/nightly/reference/macros.html#railroad-DelimTokenTree
fn collect_level(stream: &TokenStream, path: &mut Vec<usize>, out: &mut Vec<Slot>) {
    use TokenTree::*;
    use rustc_span::DUMMY_SP;

    let mut current: Vec<TokenTree> = Vec::new();
    let mut first_span: Option<Span> = None;
    let mut slot_index = 0;

    // Walk the token trees of *this* group
    for tt in stream.iter() {
        let push_tok = |tok: TokenTree, cur: &mut Vec<TokenTree>, span: &mut Option<Span>| {
            span.get_or_insert(tok.span());
            cur.push(tok);
        };

        match &tt {
            // Nested group: recurse, but keep group token in *this* slot
            Delimited(_, _, _delim, inner) => {
                push_tok(tt.clone(), &mut current, &mut first_span);
                path.push(slot_index);               // enter one level deeper
                collect_level(inner, path, out);     // recurse
                path.pop();
            }

            // Top-level comma - finish current slot
            Token(tok, _) if tok.kind == TokenKind::Comma => {
                // flush the slot we have just built (may be empty for `(.. , , ..)`)
                out.push(Slot {
                    path: path.clone(),
                    tokens: TokenStream::from_iter(current.clone()),
                    span: first_span.unwrap_or(tt.span()),
                });
                current.clear();
                first_span = None;
                slot_index += 1;
            }

            // Ordinary token
            _ => push_tok(tt.clone(), &mut current, &mut first_span),
        }
    }

    // Emit last slot of the group
    if !current.is_empty() || first_span.is_some() {
        out.push(Slot {
            path: path.clone(),
            tokens: TokenStream::from_iter(current),
            span: first_span.unwrap_or(DUMMY_SP),   // fallback span
        });
    }
}

pub fn contains_dollar_ty(stream: &TokenStream) -> bool {
    use TokenTree::*;
    use rustc_span::symbol;

    let mut it = stream.iter().peekable();

    while let Some(tt) = it.next() {
        match &tt {
            //------------------------------------------------ recurse groups
            TokenTree::Delimited(_, _, _, inner) => {
                if contains_dollar_ty(inner) {
                    return true;
                }
            }

            //------------------------------------------------ look for `$`
            TokenTree::Token(tok, _) if tok.kind == TokenKind::Dollar => {
                // Need three more tokens: Ident, Colon, Ident("ty")
                let ident  = it.next();
                let colon  = it.next();
                let ident2 = it.next();

                if let ( Some(TokenTree::Token(tok_ident, _)),
                         Some(TokenTree::Token(tok_colon, _)),
                         Some(TokenTree::Token(tok_ty,   _)) )
                    = (ident, colon, ident2)
                {
                    if matches!(tok_ident.kind, TokenKind::Ident(..))
                        && tok_colon.kind == TokenKind::Colon
                        && matches!(tok_ty.kind,
                                    TokenKind::Ident(sym, _) if sym == symbol::sym::ty)
                    {
                        return true;      // FOUND  `$whatever:ty`
                    }
                }
            }

            _ => {}
        }
    }
    false
}
