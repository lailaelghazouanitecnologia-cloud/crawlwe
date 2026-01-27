//! ZAD Abstract Syntax Tree
//!
//! Defines the structure of ZAD programs.

use std::collections::HashMap;

/// A complete ZAD program
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Top-level statements
#[derive(Debug, Clone)]
pub enum Statement {
    /// Template definition
    Template(TemplateDef),
    /// Import statement
    Import(ImportStmt),
    /// Emit directive (invoke template)
    Emit(EmitStmt),
    /// Use directive (inline template)
    Use(UseStmt),
    /// Constant definition
    Const(ConstDef),
    /// Raw code block
    Raw(String),
    /// Comment
    Comment(String),
}

/// Template definition
/// ```zad
/// template name(param: type, ...) { body }
/// ```
#[derive(Debug, Clone)]
pub struct TemplateDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: TemplateBody,
    pub doc: Option<String>,
}

/// Template parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub typ: Type,
    pub default: Option<Expr>,
}

/// Type in ZAD
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Str,
    Int,
    Float,
    Bool,
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Struct(String),
    Any,
    Optional(Box<Type>),
}

impl Type {
    pub fn from_str(s: &str) -> Self {
        match s {
            "str" => Type::Str,
            "int" => Type::Int,
            "float" => Type::Float,
            "bool" => Type::Bool,
            "any" => Type::Any,
            s if s.starts_with("[]") => {
                let inner = &s[2..];
                Type::Array(Box::new(Type::from_str(inner)))
            }
            s if s.starts_with("?") => {
                let inner = &s[1..];
                Type::Optional(Box::new(Type::from_str(inner)))
            }
            s => Type::Struct(s.to_string()),
        }
    }
}

/// Template body - contains template elements
#[derive(Debug, Clone)]
pub struct TemplateBody {
    pub elements: Vec<TemplateElement>,
}

/// Elements that can appear in a template body
#[derive(Debug, Clone)]
pub enum TemplateElement {
    /// Literal text to output
    Text(String),
    /// Expression interpolation: {expr}
    Interpolation(Expr),
    /// For loop
    For(ForLoop),
    /// If conditional
    If(IfBlock),
    /// Switch/match
    Switch(SwitchBlock),
    /// Nested template use
    Use(UseStmt),
    /// Directive: @name(args)
    Directive(Directive),
    /// Let binding
    Let(LetBinding),
}

/// For loop in template
/// ```zad
/// for (items) |item| {
///     {item.name}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ForLoop {
    pub iterable: Expr,
    pub binding: String,
    pub index_binding: Option<String>,
    pub body: TemplateBody,
    pub separator: Option<String>,
}

/// If block
/// ```zad
/// if (condition) {
///     ...
/// } else if (other) {
///     ...
/// } else {
///     ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct IfBlock {
    pub condition: Expr,
    pub then_body: TemplateBody,
    pub else_if: Vec<(Expr, TemplateBody)>,
    pub else_body: Option<TemplateBody>,
}

/// Switch/match block
/// ```zad
/// switch (value) {
///     .variant => { ... },
///     .other => |payload| { ... },
///     else => { ... },
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SwitchBlock {
    pub value: Expr,
    pub cases: Vec<SwitchCase>,
    pub default: Option<TemplateBody>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub pattern: Pattern,
    pub binding: Option<String>,
    pub body: TemplateBody,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Variant(String),
    Wildcard,
}

/// Directive
/// ```zad
/// @indent(4)
/// @newline
/// @raw("{literal}")
/// @trim
/// @emit template_name(args)
/// @use template_name(args)
/// ```
#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub args: Vec<Expr>,
}

/// Let binding
/// ```zad
/// let x = expr;
/// ```
#[derive(Debug, Clone)]
pub struct LetBinding {
    pub name: String,
    pub value: Expr,
}

/// Import statement
/// ```zad
/// import "path/to/file.zad";
/// import { template1, template2 } from "file.zad";
/// ```
#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub path: String,
    pub items: Option<Vec<String>>,
}

/// Emit statement (top-level template invocation)
/// ```zad
/// @emit template_name(arg1, arg2);
/// ```
#[derive(Debug, Clone)]
pub struct EmitStmt {
    pub template: String,
    pub args: Vec<Expr>,
}

/// Use statement (inline template)
/// ```zad
/// @use template_name(arg1, arg2);
/// ```
#[derive(Debug, Clone)]
pub struct UseStmt {
    pub template: String,
    pub args: Vec<Expr>,
}

/// Constant definition
/// ```zad
/// const NAME = value;
/// ```
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub value: Expr,
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Var(String),
    /// Field access: expr.field
    Field(Box<Expr>, String),
    /// Index access: expr[index]
    Index(Box<Expr>, Box<Expr>),
    /// Function/method call: expr(args)
    Call(Box<Expr>, Vec<Expr>),
    /// Binary operation
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// Unary operation
    Unary(UnaryOp, Box<Expr>),
    /// Array literal: [a, b, c]
    Array(Vec<Expr>),
    /// Object literal: { field: value, ... }
    Object(Vec<(String, Expr)>),
    /// Ternary: condition ? then : else
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Template string: `text {expr} more`
    TemplateString(Vec<TemplateStringPart>),
}

#[derive(Debug, Clone)]
pub enum TemplateStringPart {
    Text(String),
    Expr(Expr),
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    // String
    Concat,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
    Template(TemplateDef),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "str",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Null => "null",
            Value::Template(_) => "template",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            _ => true,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get_field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(name),
            _ => None,
        }
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(a) => {
                let items: Vec<_> = a.iter().map(|v| v.to_string_repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(o) => {
                let items: Vec<_> = o.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string_repr()))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
            Value::Template(t) => format!("<template {}>", t.name),
        }
    }
}

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::String(s) => Value::String(s),
            Literal::Int(i) => Value::Int(i),
            Literal::Float(f) => Value::Float(f),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Null => Value::Null,
        }
    }
}
