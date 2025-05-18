use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::ast::*;
use rustc_ast::tokenstream::{TokenStream, TokenTree};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::declare_lint_pass;
use rustc_span::Span;
use std::ops::Deref;
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
        if !mac_def.macro_rules {
            return;
        }

        let rules = split_into_rules(&mac_def.body.tokens.clone());

        // TODO: (PLeVasseur) - Need to get the proper span of the specialized
        // MacroMatcher; currently on the "fat arrow" (=>)
        for rule in rules {
            span_lint_and_help(
                cx,
                AVOID_SPECIALIZED_AND_GENERIC_PATTERNS_IN_DECLARATIVE_MACRO,
                rule.2,
                "avoid specialized matchers in declarative macros alongside generic ones",
                None,
                "consider implementing specialized patterns outside of macro",
            );
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

    // `trees()` yields an iterator over *borrowed* TokenTrees
    for tt in body.iter() {
        match &tt {
            // 1??  Nested groups: treat as one opaque token
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

