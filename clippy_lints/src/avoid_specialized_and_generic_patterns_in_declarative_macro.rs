use clippy_utils::diagnostics::span_lint_and_help;
use rustc_ast::ast::*;
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::declare_lint_pass;
use std::ops::Deref;

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

        // TODO: (PLeVasseur) - Need to get the proper span of the specialized
        // MacroMatcher
        span_lint_and_help(
            cx,
            AVOID_SPECIALIZED_AND_GENERIC_PATTERNS_IN_DECLARATIVE_MACRO,
            mac_def.body.deref().dspan.open,
            "avoid specialized matchers in declarative macros alongside generic ones",
            None,
            "consider implementing specialized patterns outside of macro",
        );
    }
}
