//! ZAD - Template Metaprogramming Language
//!
//! A language for generating code through composable templates.
//!
//! ## Syntax Overview
//!
//! ```zad
//! // Define a template
//! template struct_def(name: str, fields: []Field) {
//!     pub const {name} = struct {
//!         for (fields) |f| {
//!             {f.name}: {f.type},
//!         }
//!     };
//! }
//!
//! // Use a template
//! @emit struct_def("User", [
//!     { name: "id", type: "u64" },
//!     { name: "email", type: "[]const u8" },
//! ]);
//! ```

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod eval;
pub mod codegen;

pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
pub use eval::Evaluator;
pub use codegen::CodeGenerator;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZadError {
    #[error("Lexer error at line {line}: {message}")]
    LexerError { line: usize, message: String },

    #[error("Parser error at line {line}: {message}")]
    ParserError { line: usize, message: String },

    #[error("Eval error: {0}")]
    EvalError(String),

    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

pub type Result<T> = std::result::Result<T, ZadError>;

/// Compile and evaluate ZAD source code
pub fn compile(source: &str) -> Result<String> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut evaluator = Evaluator::new();
    let output = evaluator.eval(&ast)?;

    Ok(output)
}

/// Compile ZAD to target language
pub fn compile_to(source: &str, target: Target) -> Result<String> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let generator = CodeGenerator::new(target);
    let output = generator.generate(&ast)?;

    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    Zig,
    Rust,
    TypeScript,
    Python,
    Raw,  // Just output the template result
}
