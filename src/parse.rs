//! Parser for the benchmark file specification.
//! 
//! The benchmark DSL has the following grammar:
//! 
//! WhiteSpaceChar  -> ' ' | '\t' | '\n' | '\r'
//! Comment         -> ';' [^'\n']* ('\n' | EOF)
//! WhiteSpace      -> (WhiteSpaceChar | Comment)*
//! Identifier      -> !(WhiteSpace | '(' | ')' | ';')
//! Variable        -> '?' Identifier
//! IntegerLiteral  -> '-'? [0..9]+
//! StringLiteral   -> '"' [_] '"'
//! BoolLiteral     -> 'True' | 'False'
//! TermAtom        -> BoolLiteral | IntegerLiteral | StringLiteral | Variable | Identifier
//! TermList        -> '(' WhiteSpace Identifier (WhiteSpace TermAtom)* WhiteSpace ')'
//! Term            -> TermList | TermAtom
//! SortDecl        -> '(' WhiteSpace 'sort' WhiteSpace Identifier WhiteSpace ')'
//! FuncDecl        -> '(' WhiteSpace 'function' WhiteSpace Identifier WhiteSpace
//!                         '(' (WhiteSpace Identifier)* WhiteSpace ')'
//!                         WhiteSpace Identifier (WhiteSpace Identifier WhiteSpace | WhiteSpace) ')'
//! PropertyDecl    -> (' WhiteSpace 'property' WhiteSpace Identifier WhiteSpace
//!                         '(' (WhiteSpace Identifier)* WhiteSpace ')'
//!                         WhiteSpace Identifier WhiteSpace ')'
//! 
//! TODO: Support properties as well as terms
//! RewriteDecl     -> '(' WhiteSpace 'rewrite' WhiteSpace Term WhiteSpace Term WhiteSpace ')'
//!                         | '(' WhiteSpace 'birewrite' WhiteSpace Term WhiteSpace Term WhiteSpace ')'
//!                         | '(' WhiteSpace 'rewrite' WhiteSpace Term WhiteSpace Term WhiteSpace Term WhiteSpace ')'
//!                         | '(' WhiteSpace 'birewrite' WhiteSpace Term WhiteSpace Term WhiteSpace Term WhiteSpace ')'
//! Optimize        -> '(' WhiteSpace 'optimize' WhiteSpace Term WhiteSpace ')'

use crate::*;

peg::parser! {
    grammar sexp_parser() for str {
        rule ws_char()
            = [' ' | '\t' | '\n' | '\r']

        rule comment()
            = ";" [^'\n']* ("\n" / ![_])

        rule ws()
            = (ws_char() / comment())*

        rule identifier() -> String
            = s:$((!(['(' | ')' | ';'] / ws_char()) [_])+) 
                                        { s.to_string() }

        rule variable() -> String
            = s:$("?" identifier())     { s.to_string() }
        
        rule bool_lit() -> bool
            = b:$("True" / "False")     { b == "True"}

        rule int_lit() -> i64
            = n:$("-"? ['0'..='9']+)    {? n.parse().map_err(|_| "invalid integer") }

        rule string_lit() -> String
            = "\"" s:$([^'\"']*) "\""   { s.to_string() }

        rule term_atom() -> Term
            = b:bool_lit()              { Term::BoolLit(b) }
            / n:int_lit()               { Term::IntLit(n) }
            / s:string_lit()            { Term::Call(s, vec![]) }
            / v:variable()              { Term::Var(v) }
            / i:identifier()            { Term::Identifier(i) }

        rule term_list() -> Term
            = "(" ws() f:identifier() args:(ws() t:term() { t })* ws() ")" {
                Term::Call(f, args)
            }

        rule term() -> Term
            = ws() t:(term_list() / term_atom()) ws() { t }

        rule sort_decl() -> Decl
            = "(" ws() "sort" ws() name:identifier() ws() ")" {
                Decl::Sort(Sort { name })
            }

        rule function_decl() -> Decl
            = "(" ws() "function" ws() name:identifier() ws()
              "(" args:(ws() a:identifier() { a })* ws() ")" ws()
              ret:identifier() ws() ")" {
                Decl::Function(Function { name, args, ret })
            }

        rule rewrite_decl() -> Decl
            = "(" ws() "rewrite" ws() name:identifier() ws() lhs:term() ws() rhs:term() ws() ")" {
                Decl::Rewrite(Rewrite { name, lhs, rhs })
            }
            / "(" ws() "rewrite" ws() lhs:term() ws() rhs:term() ws() ")" {
                Decl::Rewrite(Rewrite { name: String::new(), lhs, rhs })
            }

        rule optimize_decl() -> Decl
            = "(" ws() "optimize" ws() term:term() ws() ")" {
                Decl::Optimize(Optimize { term })
            }

        rule decl() -> Decl
            = sort_decl() / function_decl() / rewrite_decl() / optimize_decl()

        pub rule parse_term() -> Term
            = ws() t:(term_list() / term_atom()) ws() ![_] { t }

        pub rule parse_decl() -> Decl
            = ws() d:decl() ws() ![_] { d }

        pub rule parse_decls() -> Vec<Decl>
            = ws() ds:(decl() ** ws()) ws() ![_] { ds }
    }
}

pub fn parse_term(input: &str) -> Result<Term> {
    sexp_parser::parse_term(input).map_err(|e| e.to_string())
}

pub fn parse_decl(input: &str) -> Result<Decl> {
    sexp_parser::parse_decl(input).map_err(|e| e.to_string())
}

pub fn parse_decls(input: &str) -> Result<Vec<Decl>> {
    sexp_parser::parse_decls(input).map_err(|e| e.to_string())
}
