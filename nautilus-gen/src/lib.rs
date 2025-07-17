use std::{
    any::{Any, TypeId},
    collections::HashSet,
    fmt::{Display, write},
};

use proc_macro::TokenStream;
use proc_macro2::{Literal, Punct, TokenTree};
use quote::quote;
use syn::{Ident, LitBool, LitByteStr, LitCStr, LitStr, Token, parse::Parse, parse_macro_input};
#[cfg(test)]
mod tests;

struct NautilusGrammar {
    rules: Vec<Rule>,
}
fn check_for_unused_rule_names(grammar: &NautilusGrammar) -> Result<(), Vec<&Name>> {
    let rule_names_map: HashSet<&Name> = grammar.rules.iter().map(|r| &r.name).collect();
    let mut names_without_rule: Vec<&Name> = Vec::new();

    // Checks each non terminating symbol to see if it has a rule associated with it. If it
    // doesn't, then it gets returned in the Err variant of the Result
    for expression in grammar.rules.iter().map(|r| &r.expr) {
        for symbol in expression.0.iter() {
            if let Symbol::Nonterminating(nonterm) = symbol {
                if !rule_names_map.contains(nonterm) {
                    names_without_rule.push(nonterm);
                }
            }
        }
    }

    if names_without_rule.is_empty() {
        return Ok(());
    }

    Err(names_without_rule)
}

impl Parse for NautilusGrammar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules = Vec::new();
        while !input.is_empty() {
            rules.push(input.parse()?);
        }
        let grammar = NautilusGrammar { rules };
        if let Err(without_rules) = check_for_unused_rule_names(&grammar) {
            let mut repr = String::new();
            for name in without_rules {
                repr.push_str(name.to_string().as_str());
                repr.push_str(", ");
            }
            panic!(
                "The following names don't have associated rules:\n{:?}",
                repr
            );
        }
        Ok(grammar)
    }
}

/// This represents each rule of the
/// nautilus grammar
struct Rule {
    name: Name,
    expr: Expr,
}

impl Parse for Rule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Name = input.parse()?;
        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { name, expr })
    }
}

/// This represents the name of the rule
#[derive(Hash, PartialEq, Eq)]
struct Name(Ident);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Parse for Name {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        if name == "true" || name == "false" {
            return Err(input.error("Cannot have true or false as a rule name"));
        }

        Ok(Self(name))
    }
}

/// This represents the expression that a `Name` corresponds to
struct Expr(Vec<Symbol>);

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for symbol in self.0.iter() {
            match symbol {
                Symbol::Terminating(terminating) => s.push_str(terminating.to_string().as_str()),
                Symbol::Nonterminating(name) => s.push_str(format!("{{{}}}", name).as_str()),
            }
        }

        write!(f, "{}", s)
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut expression: Vec<Symbol> = Vec::new();
        while let Ok(symbol) = input.parse() {
            expression.push(symbol);
        }
        Ok(Expr(expression))
    }
}

/// The parts that an `Expr` are made of
enum Symbol {
    Terminating(Terminating),
    Nonterminating(Name),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Terminating(terminating) => write!(f, "{}", terminating),
            Symbol::Nonterminating(name) => write!(f, "{}", name),
        }
    }
}

impl Parse for Symbol {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(terminating) = input.parse() {
            return Ok(Symbol::Terminating(terminating));
        }
        let nonterm = input.parse()?;
        Ok(Symbol::Nonterminating(nonterm))
    }
}

enum Terminating {
    Lit(Literal),
    Operator(String),
    LitBool(LitBool),
}

impl Display for Terminating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terminating::Lit(literal) => {
                let repr = literal.to_string();
                if repr.starts_with("\"") && repr.ends_with("\"") {
                    return write!(f, "{}", &repr[1..repr.len() - 1]);
                }
                write!(f, "{}", literal)
            }
            Terminating::LitBool(lit_bool) => {
                if lit_bool.value() {
                    write!(f, "{}", true)
                } else {
                    write!(f, "{}", false)
                }
            }
            Terminating::Operator(s) => write!(f, "{}", s),
        }
    }
}

impl Parse for Terminating {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse() {
            return Ok(Terminating::Lit(lit));
        }
        if let Ok(lit) = input.parse() {
            return Ok(Terminating::LitBool(lit));
        }

        //Parse non alphanumeric strings of punctuation as an operator
        let fork = input.fork();

        let mut op = String::new();

        while let Ok(TokenTree::Punct(p)) = fork.parse() {
            if p.as_char() == ';' {
                break;
            }

            op.push(p.as_char());
        }

        if !op.is_empty() {
            // Advance the main parser by the amount parsed using fork
            for _ in 0..op.len() {
                input.parse::<Punct>()?;
            }
            return Ok(Terminating::Operator(op));
        }
        Err(input.error("Could not parse terminating symbol"))
    }
}

/// This only has to parse to a vec that libafl::generators::nautilus::NautilusContext::with_rules
/// considers valid
#[proc_macro]
pub fn nautilus_gen(input: TokenStream) -> TokenStream {
    let grammar = parse_macro_input!(input as NautilusGrammar);

    let rules: Vec<_> = grammar
        .rules
        .iter()
        .map(|rule| {
            let name = &rule.name.to_string();
            let expression_str = rule.expr.to_string();
            let expr_bytes = expression_str.as_bytes();

            quote! {
                (#name, #expression_str.as_bytes())
            }
        })
        .collect();

    TokenStream::from(quote! {
        vec![#(#rules),*]
    })
}
