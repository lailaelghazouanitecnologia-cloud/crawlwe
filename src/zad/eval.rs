//! ZAD Evaluator - Executes templates and produces output

use std::collections::HashMap;
use crate::zad::ast::*;
use crate::zad::{ZadError, Result};

pub struct Evaluator {
    templates: HashMap<String, TemplateDef>,
    constants: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    output: String,
    indent_level: usize,
    indent_size: usize,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            constants: HashMap::new(),
            scopes: vec![HashMap::new()],
            output: String::new(),
            indent_level: 0,
            indent_size: 4,
        }
    }

    pub fn eval(&mut self, program: &Program) -> Result<String> {
        // First pass: collect templates and constants
        for stmt in &program.statements {
            match stmt {
                Statement::Template(template) => {
                    self.templates.insert(template.name.clone(), template.clone());
                }
                Statement::Const(def) => {
                    let value = self.eval_expr(&def.value)?;
                    self.constants.insert(def.name.clone(), value);
                }
                _ => {}
            }
        }

        // Second pass: execute emit statements
        for stmt in &program.statements {
            self.eval_statement(stmt)?;
        }

        Ok(std::mem::take(&mut self.output))
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Template(_) | Statement::Const(_) | Statement::Import(_) => {
                // Already processed or ignored
                Ok(())
            }
            Statement::Emit(emit) => {
                self.eval_emit(emit)
            }
            Statement::Use(use_stmt) => {
                self.eval_use(use_stmt)
            }
            Statement::Raw(text) => {
                self.emit(text);
                Ok(())
            }
            Statement::Comment(_) => Ok(()),
        }
    }

    fn eval_emit(&mut self, emit: &EmitStmt) -> Result<()> {
        let template = self.templates.get(&emit.template)
            .ok_or_else(|| ZadError::TemplateNotFound(emit.template.clone()))?
            .clone();

        let args = emit.args.iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<Vec<_>>>()?;

        self.call_template(&template, args)
    }

    fn eval_use(&mut self, use_stmt: &UseStmt) -> Result<()> {
        let template = self.templates.get(&use_stmt.template)
            .ok_or_else(|| ZadError::TemplateNotFound(use_stmt.template.clone()))?
            .clone();

        let args = use_stmt.args.iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<Vec<_>>>()?;

        self.call_template(&template, args)
    }

    fn call_template(&mut self, template: &TemplateDef, args: Vec<Value>) -> Result<()> {
        // Create new scope with parameters
        let mut scope = HashMap::new();

        for (i, param) in template.params.iter().enumerate() {
            let value = args.get(i)
                .cloned()
                .or_else(|| param.default.as_ref().and_then(|d| self.eval_expr(d).ok()))
                .ok_or_else(|| ZadError::EvalError(
                    format!("Missing argument: {}", param.name)
                ))?;
            scope.insert(param.name.clone(), value);
        }

        self.scopes.push(scope);
        self.eval_template_body(&template.body)?;
        self.scopes.pop();

        Ok(())
    }

    fn eval_template_body(&mut self, body: &TemplateBody) -> Result<()> {
        for element in &body.elements {
            self.eval_template_element(element)?;
        }
        Ok(())
    }

    fn eval_template_element(&mut self, element: &TemplateElement) -> Result<()> {
        match element {
            TemplateElement::Text(text) => {
                self.emit(text);
            }
            TemplateElement::Interpolation(expr) => {
                let value = self.eval_expr(expr)?;
                self.emit(&value.to_string_repr());
            }
            TemplateElement::For(for_loop) => {
                self.eval_for_loop(for_loop)?;
            }
            TemplateElement::If(if_block) => {
                self.eval_if_block(if_block)?;
            }
            TemplateElement::Switch(switch_block) => {
                self.eval_switch_block(switch_block)?;
            }
            TemplateElement::Use(use_stmt) => {
                self.eval_use(use_stmt)?;
            }
            TemplateElement::Directive(directive) => {
                self.eval_directive(directive)?;
            }
            TemplateElement::Let(binding) => {
                let value = self.eval_expr(&binding.value)?;
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(binding.name.clone(), value);
                }
            }
        }
        Ok(())
    }

    fn eval_for_loop(&mut self, for_loop: &ForLoop) -> Result<()> {
        let iterable = self.eval_expr(&for_loop.iterable)?;

        let items = match iterable {
            Value::Array(arr) => arr,
            Value::Object(obj) => {
                // Iterate over key-value pairs
                obj.into_iter()
                    .map(|(k, v)| {
                        let mut pair = HashMap::new();
                        pair.insert("key".to_string(), Value::String(k));
                        pair.insert("value".to_string(), v);
                        Value::Object(pair)
                    })
                    .collect()
            }
            Value::String(s) => {
                // Iterate over characters
                s.chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect()
            }
            _ => {
                return Err(ZadError::TypeError {
                    expected: "iterable".into(),
                    got: iterable.type_name().into(),
                });
            }
        };

        let total = items.len();

        for (index, item) in items.into_iter().enumerate() {
            // Add separator before non-first items
            if index > 0 {
                if let Some(sep) = &for_loop.separator {
                    self.emit(sep);
                }
            }

            // Create scope with loop variable
            let mut scope = HashMap::new();
            scope.insert(for_loop.binding.clone(), item);

            if let Some(idx_name) = &for_loop.index_binding {
                scope.insert(idx_name.clone(), Value::Int(index as i64));
            }

            // Add special loop variables
            scope.insert("@index".to_string(), Value::Int(index as i64));
            scope.insert("@first".to_string(), Value::Bool(index == 0));
            scope.insert("@last".to_string(), Value::Bool(index == total - 1));

            self.scopes.push(scope);
            self.eval_template_body(&for_loop.body)?;
            self.scopes.pop();
        }

        Ok(())
    }

    fn eval_if_block(&mut self, if_block: &IfBlock) -> Result<()> {
        let condition = self.eval_expr(&if_block.condition)?;

        if condition.is_truthy() {
            self.eval_template_body(&if_block.then_body)?;
            return Ok(());
        }

        for (cond, body) in &if_block.else_if {
            let value = self.eval_expr(cond)?;
            if value.is_truthy() {
                self.eval_template_body(body)?;
                return Ok(());
            }
        }

        if let Some(else_body) = &if_block.else_body {
            self.eval_template_body(else_body)?;
        }

        Ok(())
    }

    fn eval_switch_block(&mut self, switch_block: &SwitchBlock) -> Result<()> {
        let value = self.eval_expr(&switch_block.value)?;

        for case in &switch_block.cases {
            let matches = match &case.pattern {
                Pattern::Literal(lit) => {
                    let lit_val: Value = lit.clone().into();
                    self.values_equal(&value, &lit_val)
                }
                Pattern::Variant(variant) => {
                    if let Value::String(s) = &value {
                        s == variant
                    } else {
                        false
                    }
                }
                Pattern::Wildcard => true,
            };

            if matches {
                if let Some(binding) = &case.binding {
                    let mut scope = HashMap::new();
                    scope.insert(binding.clone(), value.clone());
                    self.scopes.push(scope);
                    self.eval_template_body(&case.body)?;
                    self.scopes.pop();
                } else {
                    self.eval_template_body(&case.body)?;
                }
                return Ok(());
            }
        }

        if let Some(default) = &switch_block.default {
            self.eval_template_body(default)?;
        }

        Ok(())
    }

    fn eval_directive(&mut self, directive: &Directive) -> Result<()> {
        match directive.name.as_str() {
            "indent" => {
                if let Some(arg) = directive.args.first() {
                    let level = self.eval_expr(arg)?;
                    if let Value::Int(n) = level {
                        self.indent_level = n as usize;
                    }
                } else {
                    self.indent_level += 1;
                }
            }
            "dedent" => {
                self.indent_level = self.indent_level.saturating_sub(1);
            }
            "newline" | "nl" => {
                self.emit("\n");
            }
            "space" => {
                self.emit(" ");
            }
            "raw" => {
                if let Some(arg) = directive.args.first() {
                    let text = self.eval_expr(arg)?;
                    if let Value::String(s) = text {
                        self.output.push_str(&s);
                    }
                }
            }
            "trim" => {
                // Trim trailing whitespace from output
                while self.output.ends_with(' ') || self.output.ends_with('\t') {
                    self.output.pop();
                }
            }
            "emit_debug_info" => {
                self.emit(&format!("// Debug: {} templates, {} scopes\n",
                    self.templates.len(), self.scopes.len()));
            }
            _ => {
                // Unknown directive - ignore or warn
            }
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(lit.clone().into()),

            Expr::Var(name) => self.lookup_var(name),

            Expr::Field(obj, field) => {
                let obj_val = self.eval_expr(obj)?;
                match obj_val {
                    Value::Object(map) => {
                        map.get(field).cloned()
                            .ok_or_else(|| ZadError::EvalError(
                                format!("Field '{}' not found", field)
                            ))
                    }
                    _ => Err(ZadError::TypeError {
                        expected: "object".into(),
                        got: obj_val.type_name().into(),
                    }),
                }
            }

            Expr::Index(arr, idx) => {
                let arr_val = self.eval_expr(arr)?;
                let idx_val = self.eval_expr(idx)?;

                match (arr_val, idx_val) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let i = if i < 0 { arr.len() as i64 + i } else { i } as usize;
                        arr.get(i).cloned()
                            .ok_or_else(|| ZadError::EvalError(
                                format!("Index {} out of bounds", i)
                            ))
                    }
                    (Value::Object(map), Value::String(key)) => {
                        map.get(&key).cloned()
                            .ok_or_else(|| ZadError::EvalError(
                                format!("Key '{}' not found", key)
                            ))
                    }
                    (Value::String(s), Value::Int(i)) => {
                        let i = if i < 0 { s.len() as i64 + i } else { i } as usize;
                        s.chars().nth(i)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| ZadError::EvalError(
                                format!("Index {} out of bounds", i)
                            ))
                    }
                    _ => Err(ZadError::EvalError("Invalid index operation".into())),
                }
            }

            Expr::Call(func, args) => {
                let func_name = match func.as_ref() {
                    Expr::Var(name) => name.clone(),
                    _ => return Err(ZadError::EvalError("Expected function name".into())),
                };

                let arg_vals = args.iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<Vec<_>>>()?;

                self.call_builtin(&func_name, arg_vals)
            }

            Expr::Binary(left, op, right) => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary_op(&left_val, *op, &right_val)
            }

            Expr::Unary(op, expr) => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                    UnaryOp::Neg => match val {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(ZadError::TypeError {
                            expected: "number".into(),
                            got: val.type_name().into(),
                        }),
                    },
                }
            }

            Expr::Array(elements) => {
                let values = elements.iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Value::Array(values))
            }

            Expr::Object(fields) => {
                let mut map = HashMap::new();
                for (key, expr) in fields {
                    let value = self.eval_expr(expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }

            Expr::Ternary(cond, then_expr, else_expr) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }

            Expr::TemplateString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        TemplateStringPart::Text(text) => result.push_str(text),
                        TemplateStringPart::Expr(expr) => {
                            let val = self.eval_expr(expr)?;
                            result.push_str(&val.to_string_repr());
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }

    fn lookup_var(&self, name: &str) -> Result<Value> {
        // Check local scopes (innermost first)
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check constants
        if let Some(value) = self.constants.get(name) {
            return Ok(value.clone());
        }

        Err(ZadError::UndefinedVariable(name.to_string()))
    }

    fn call_builtin(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        match name {
            "len" | "length" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("len requires 1 argument".into()))?;
                match arg {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::Array(a) => Ok(Value::Int(a.len() as i64)),
                    Value::Object(o) => Ok(Value::Int(o.len() as i64)),
                    _ => Err(ZadError::TypeError {
                        expected: "string, array, or object".into(),
                        got: arg.type_name().into(),
                    }),
                }
            }

            "upper" | "uppercase" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("upper requires 1 argument".into()))?;
                match arg {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    _ => Err(ZadError::TypeError {
                        expected: "string".into(),
                        got: arg.type_name().into(),
                    }),
                }
            }

            "lower" | "lowercase" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("lower requires 1 argument".into()))?;
                match arg {
                    Value::String(s) => Ok(Value::String(s.to_lowercase())),
                    _ => Err(ZadError::TypeError {
                        expected: "string".into(),
                        got: arg.type_name().into(),
                    }),
                }
            }

            "trim" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("trim requires 1 argument".into()))?;
                match arg {
                    Value::String(s) => Ok(Value::String(s.trim().to_string())),
                    _ => Err(ZadError::TypeError {
                        expected: "string".into(),
                        got: arg.type_name().into(),
                    }),
                }
            }

            "join" => {
                let mut args = args.into_iter();
                let arr = args.next()
                    .ok_or_else(|| ZadError::EvalError("join requires 2 arguments".into()))?;
                let sep = args.next()
                    .map(|v| match v {
                        Value::String(s) => s,
                        _ => ", ".to_string(),
                    })
                    .unwrap_or_else(|| ", ".to_string());

                match arr {
                    Value::Array(a) => {
                        let strings: Vec<String> = a.iter()
                            .map(|v| v.to_string_repr())
                            .collect();
                        Ok(Value::String(strings.join(&sep)))
                    }
                    _ => Err(ZadError::TypeError {
                        expected: "array".into(),
                        got: arr.type_name().into(),
                    }),
                }
            }

            "split" => {
                let mut args = args.into_iter();
                let s = args.next()
                    .ok_or_else(|| ZadError::EvalError("split requires 2 arguments".into()))?;
                let sep = args.next()
                    .map(|v| match v {
                        Value::String(s) => s,
                        _ => " ".to_string(),
                    })
                    .unwrap_or_else(|| " ".to_string());

                match s {
                    Value::String(s) => {
                        let parts: Vec<Value> = s.split(&sep)
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Ok(Value::Array(parts))
                    }
                    _ => Err(ZadError::TypeError {
                        expected: "string".into(),
                        got: s.type_name().into(),
                    }),
                }
            }

            "typeof" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("typeof requires 1 argument".into()))?;
                Ok(Value::String(arg.type_name().to_string()))
            }

            "str" | "string" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("str requires 1 argument".into()))?;
                Ok(Value::String(arg.to_string_repr()))
            }

            "int" => {
                let arg = args.into_iter().next()
                    .ok_or_else(|| ZadError::EvalError("int requires 1 argument".into()))?;
                match arg {
                    Value::Int(i) => Ok(Value::Int(i)),
                    Value::Float(f) => Ok(Value::Int(f as i64)),
                    Value::String(s) => s.parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| ZadError::EvalError(format!("Cannot convert '{}' to int", s))),
                    Value::Bool(b) => Ok(Value::Int(if b { 1 } else { 0 })),
                    _ => Err(ZadError::TypeError {
                        expected: "convertible to int".into(),
                        got: arg.type_name().into(),
                    }),
                }
            }

            "range" => {
                let mut args = args.into_iter();
                let start = args.next()
                    .and_then(|v| v.as_int())
                    .ok_or_else(|| ZadError::EvalError("range requires int arguments".into()))?;
                let end = args.next()
                    .and_then(|v| v.as_int())
                    .unwrap_or(start);
                let step = args.next()
                    .and_then(|v| v.as_int())
                    .unwrap_or(1);

                let (start, end) = if args.next().is_none() && end == start {
                    (0, start)
                } else {
                    (start, end)
                };

                let values: Vec<Value> = (start..end)
                    .step_by(step.max(1) as usize)
                    .map(Value::Int)
                    .collect();
                Ok(Value::Array(values))
            }

            _ => Err(ZadError::EvalError(format!("Unknown function: {}", name))),
        }
    }

    fn eval_binary_op(&self, left: &Value, op: BinOp, right: &Value) -> Result<Value> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(ZadError::EvalError("Cannot add these types".into())),
            },
            BinOp::Sub => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(ZadError::EvalError("Cannot subtract these types".into())),
            },
            BinOp::Mul => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                (Value::String(s), Value::Int(n)) => Ok(Value::String(s.repeat(*n as usize))),
                _ => Err(ZadError::EvalError("Cannot multiply these types".into())),
            },
            BinOp::Div => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err(ZadError::EvalError("Division by zero".into()))
                    } else {
                        Ok(Value::Int(a / b))
                    }
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err(ZadError::EvalError("Cannot divide these types".into())),
            },
            BinOp::Mod => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(ZadError::EvalError("Modulo requires integers".into())),
            },
            BinOp::Eq => Ok(Value::Bool(self.values_equal(left, right))),
            BinOp::Ne => Ok(Value::Bool(!self.values_equal(left, right))),
            BinOp::Lt => self.compare_values(left, right, |a, b| a < b),
            BinOp::Le => self.compare_values(left, right, |a, b| a <= b),
            BinOp::Gt => self.compare_values(left, right, |a, b| a > b),
            BinOp::Ge => self.compare_values(left, right, |a, b| a >= b),
            BinOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
            BinOp::Concat => {
                let left_str = left.to_string_repr();
                let right_str = right.to_string_repr();
                Ok(Value::String(format!("{}{}", left_str, right_str)))
            }
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> Result<Value>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::Bool(cmp(a.cmp(b) as i64, 0)))
            }
            _ => Err(ZadError::EvalError("Cannot compare these types".into())),
        }
    }

    fn emit(&mut self, text: &str) {
        // Handle indentation for new lines
        for line in text.split('\n') {
            if !self.output.is_empty() && self.output.ends_with('\n') && !line.is_empty() {
                self.output.push_str(&" ".repeat(self.indent_level * self.indent_size));
            }
            self.output.push_str(line);
            if text.contains('\n') && line != text.split('\n').last().unwrap_or("") {
                self.output.push('\n');
            }
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
