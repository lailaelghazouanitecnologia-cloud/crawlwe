//! VM Executor - Executes bytecode programs
//!
//! Features:
//! - Stack-based execution
//! - Pluggable parser backends (like Bun)
//! - Fast opcode dispatch

use std::collections::HashMap;
use regex::Regex;

use super::opcodes::{OpCode, Instruction, Value, Program};
use super::parser_registry::ParserRegistry;

/// VM execution state
#[derive(Debug)]
pub struct VMState {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub results: HashMap<String, Vec<Value>>,
    pub ip: usize,
    pub running: bool,
    pub source: String,
    pub call_stack: Vec<usize>,
}

impl Default for VMState {
    fn default() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            variables: HashMap::new(),
            results: HashMap::new(),
            ip: 0,
            running: true,
            source: String::new(),
            call_stack: Vec::new(),
        }
    }
}

/// VM execution error
#[derive(Debug, thiserror::Error)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Invalid operand: {0}")]
    InvalidOperand(String),

    #[error("Invalid opcode at {0}")]
    InvalidOpcode(usize),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Regex error: {0}")]
    RegexError(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },
}

/// Virtual Machine for parsing operations
pub struct VM {
    pub state: VMState,
    pub parsers: ParserRegistry,
    pub debug: bool,
    regex_cache: HashMap<String, Regex>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            state: VMState::default(),
            parsers: ParserRegistry::new(),
            debug: false,
            regex_cache: HashMap::new(),
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Execute a program with given source
    pub fn execute(&mut self, program: &Program, source: &str) -> Result<VMState, VMError> {
        self.state = VMState::default();
        self.state.source = source.to_string();

        while self.state.running && self.state.ip < program.instructions.len() {
            let instr = &program.instructions[self.state.ip];

            if self.debug {
                eprintln!("[VM] {:04} {}", self.state.ip, instr);
            }

            self.execute_instruction(instr, program)?;
            self.state.ip += 1;
        }

        Ok(std::mem::take(&mut self.state))
    }

    fn execute_instruction(&mut self, instr: &Instruction, program: &Program) -> Result<(), VMError> {
        match instr.opcode {
            // Stack operations
            OpCode::Nop => {}
            OpCode::Halt => self.state.running = false,
            OpCode::Push => {
                if let Some(v) = instr.operands.first() {
                    self.state.stack.push(v.clone());
                }
            }
            OpCode::Pop => { self.pop()?; }
            OpCode::Dup => {
                let v = self.peek()?.clone();
                self.state.stack.push(v);
            }
            OpCode::Swap => {
                let len = self.state.stack.len();
                if len < 2 {
                    return Err(VMError::StackUnderflow);
                }
                self.state.stack.swap(len - 1, len - 2);
            }
            OpCode::Rot => {
                let len = self.state.stack.len();
                if len < 3 {
                    return Err(VMError::StackUnderflow);
                }
                let top = self.state.stack.pop().unwrap();
                self.state.stack.insert(len - 3, top);
            }

            // Load/Store
            OpCode::LoadSource => {
                self.state.stack.push(Value::String(self.state.source.clone()));
            }
            OpCode::LoadConst => {
                let idx = self.get_operand_int(instr, 0)? as usize;
                if idx < program.constants.len() {
                    self.state.stack.push(program.constants[idx].clone());
                } else {
                    return Err(VMError::InvalidOperand(format!("constant index {}", idx)));
                }
            }
            OpCode::LoadVar => {
                let name = self.get_operand_string(instr, 0)?;
                let value = self.state.variables.get(&name)
                    .cloned()
                    .unwrap_or(Value::Null);
                self.state.stack.push(value);
            }
            OpCode::StoreVar => {
                let name = self.get_operand_string(instr, 0)?;
                let value = self.pop()?;
                self.state.variables.insert(name, value);
            }
            OpCode::LoadResult => {
                let name = self.get_operand_string(instr, 0)?;
                let results = self.state.results.get(&name)
                    .cloned()
                    .unwrap_or_default();
                self.state.stack.push(Value::List(results));
            }
            OpCode::StoreResult => {
                let name = self.get_operand_string(instr, 0)?;
                let value = self.pop()?;
                self.state.results.entry(name).or_default().push(value);
            }

            // Pattern matching
            OpCode::MatchRegex => {
                let pattern = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                let result = {
                    let regex = self.get_or_compile_regex(&pattern)?;
                    regex.is_match(&text)
                };
                self.state.stack.push(Value::Bool(result));
            }
            OpCode::MatchAll => {
                let pattern = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                let matches = {
                    let regex = self.get_or_compile_regex(&pattern)?;
                    regex.find_iter(&text)
                        .map(|m| Value::String(m.as_str().to_string()))
                        .collect::<Vec<Value>>()
                };
                self.state.stack.push(Value::List(matches));
            }
            OpCode::MatchFirst => {
                let pattern = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                let result = {
                    let regex = self.get_or_compile_regex(&pattern)?;
                    regex.find(&text)
                        .map(|m| Value::String(m.as_str().to_string()))
                        .unwrap_or(Value::Null)
                };
                self.state.stack.push(result);
            }
            OpCode::ExtractGroup => {
                let group = self.get_operand_int(instr, 0)? as usize;
                let pattern = self.get_operand_string(instr, 1)?;
                let text = self.pop_string()?;
                let result = {
                    let regex = self.get_or_compile_regex(&pattern)?;
                    regex.captures(&text)
                        .and_then(|c| c.get(group))
                        .map(|m| Value::String(m.as_str().to_string()))
                        .unwrap_or(Value::Null)
                };
                self.state.stack.push(result);
            }
            OpCode::TestPattern => {
                let pattern = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                let result = {
                    let regex = self.get_or_compile_regex(&pattern)?;
                    regex.is_match(&text)
                };
                self.state.stack.push(Value::Bool(result));
            }

            // String operations
            OpCode::Concat => {
                let b = self.pop_string()?;
                let a = self.pop_string()?;
                self.state.stack.push(Value::String(format!("{}{}", a, b)));
            }
            OpCode::Split => {
                let delim = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                let parts: Vec<Value> = text.split(&delim)
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                self.state.stack.push(Value::List(parts));
            }
            OpCode::Trim => {
                let text = self.pop_string()?;
                self.state.stack.push(Value::String(text.trim().to_string()));
            }
            OpCode::Lower => {
                let text = self.pop_string()?;
                self.state.stack.push(Value::String(text.to_lowercase()));
            }
            OpCode::Upper => {
                let text = self.pop_string()?;
                self.state.stack.push(Value::String(text.to_uppercase()));
            }
            OpCode::Contains => {
                let needle = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                self.state.stack.push(Value::Bool(text.contains(&needle)));
            }
            OpCode::StartsWith => {
                let prefix = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                self.state.stack.push(Value::Bool(text.starts_with(&prefix)));
            }
            OpCode::EndsWith => {
                let suffix = self.get_operand_string(instr, 0)?;
                let text = self.pop_string()?;
                self.state.stack.push(Value::Bool(text.ends_with(&suffix)));
            }

            // Collection operations
            OpCode::ListNew => {
                self.state.stack.push(Value::List(Vec::new()));
            }
            OpCode::ListAppend => {
                let item = self.pop()?;
                let list = self.pop()?;
                if let Value::List(mut l) = list {
                    l.push(item);
                    self.state.stack.push(Value::List(l));
                } else {
                    return Err(VMError::TypeError {
                        expected: "list".into(),
                        got: format!("{:?}", list),
                    });
                }
            }
            OpCode::ListGet => {
                let idx = self.get_operand_int(instr, 0)? as usize;
                let list = self.pop()?;
                if let Value::List(l) = list {
                    let item = l.get(idx).cloned().unwrap_or(Value::Null);
                    self.state.stack.push(item);
                } else {
                    return Err(VMError::TypeError {
                        expected: "list".into(),
                        got: format!("{:?}", list),
                    });
                }
            }
            OpCode::ListLen => {
                let list = self.pop()?;
                if let Value::List(l) = list {
                    self.state.stack.push(Value::Int(l.len() as i64));
                } else {
                    return Err(VMError::TypeError {
                        expected: "list".into(),
                        got: format!("{:?}", list),
                    });
                }
            }
            OpCode::MapNew => {
                self.state.stack.push(Value::Map(HashMap::new()));
            }
            OpCode::MapSet => {
                let key = self.get_operand_string(instr, 0)?;
                let value = self.pop()?;
                let map = self.pop()?;
                if let Value::Map(mut m) = map {
                    m.insert(key, value);
                    self.state.stack.push(Value::Map(m));
                } else {
                    return Err(VMError::TypeError {
                        expected: "map".into(),
                        got: format!("{:?}", map),
                    });
                }
            }
            OpCode::MapGet => {
                let key = self.get_operand_string(instr, 0)?;
                let map = self.pop()?;
                if let Value::Map(m) = map {
                    let value = m.get(&key).cloned().unwrap_or(Value::Null);
                    self.state.stack.push(value);
                } else {
                    return Err(VMError::TypeError {
                        expected: "map".into(),
                        got: format!("{:?}", map),
                    });
                }
            }

            // Control flow
            OpCode::Jump => {
                let target = self.resolve_jump_target(instr, program)?;
                self.state.ip = target.wrapping_sub(1); // Will be incremented after
            }
            OpCode::JumpIf => {
                let cond = self.pop()?;
                if cond.is_truthy() {
                    let target = self.resolve_jump_target(instr, program)?;
                    self.state.ip = target.wrapping_sub(1);
                }
            }
            OpCode::JumpNot => {
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    let target = self.resolve_jump_target(instr, program)?;
                    self.state.ip = target.wrapping_sub(1);
                }
            }
            OpCode::Call => {
                let target = self.resolve_jump_target(instr, program)?;
                self.state.call_stack.push(self.state.ip);
                self.state.ip = target.wrapping_sub(1);
            }
            OpCode::Return => {
                if let Some(ret_addr) = self.state.call_stack.pop() {
                    self.state.ip = ret_addr;
                } else {
                    self.state.running = false;
                }
            }

            // Comparison
            OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.state.stack.push(Value::Bool(a == b));
            }
            OpCode::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.state.stack.push(Value::Bool(a != b));
            }
            OpCode::Lt => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Bool(a < b));
            }
            OpCode::Le => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Bool(a <= b));
            }
            OpCode::Gt => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Bool(a > b));
            }
            OpCode::Ge => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Bool(a >= b));
            }
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.state.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.state.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            OpCode::Not => {
                let v = self.pop()?;
                self.state.stack.push(Value::Bool(!v.is_truthy()));
            }
            OpCode::Add => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Int(a + b));
            }
            OpCode::Sub => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Int(a - b));
            }
            OpCode::Mul => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                self.state.stack.push(Value::Int(a * b));
            }
            OpCode::Div => {
                let b = self.pop_int()?;
                let a = self.pop_int()?;
                if b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.state.stack.push(Value::Int(a / b));
            }

            // Emit
            OpCode::Emit => {
                let value = self.pop()?;
                self.state.results.entry("default".into()).or_default().push(value);
            }
            OpCode::EmitTagged => {
                let tag = self.get_operand_string(instr, 0)?;
                let value = self.pop()?;
                self.state.results.entry(tag).or_default().push(value);
            }
            OpCode::Collect => {
                let name = self.get_operand_string(instr, 0)?;
                let value = self.pop()?;
                self.state.results.entry(name).or_default().push(value);
            }

            // Parser dispatch
            OpCode::UseParser => {
                let parser_name = self.get_operand_string(instr, 0)?;
                self.parsers.set_current(&parser_name);
            }
            OpCode::ParseWith => {
                let parser_name = self.get_operand_string(instr, 0)?;
                let source = self.pop_string()?;
                let result = self.parsers.parse(&parser_name, &source)?;
                self.state.stack.push(result);
            }

            // CSS specific
            OpCode::CssFindKeyframes => {
                let css = self.pop_string()?;
                let keyframes = self.parsers.css_find_keyframes(&css)?;
                self.state.stack.push(Value::List(keyframes));
            }
            OpCode::CssFindVars => {
                let css = self.pop_string()?;
                let vars = self.parsers.css_find_vars(&css)?;
                self.state.stack.push(Value::List(vars));
            }
            OpCode::CssFindColors => {
                let css = self.pop_string()?;
                let colors = self.parsers.css_find_colors(&css)?;
                self.state.stack.push(Value::List(colors));
            }

            // HTML specific
            OpCode::HtmlParse => {
                let html = self.pop_string()?;
                let dom = self.parsers.html_parse(&html)?;
                self.state.stack.push(dom);
            }
            OpCode::HtmlQuerySelector => {
                let selector = self.get_operand_string(instr, 0)?;
                let html = self.pop_string()?;
                let result = self.parsers.html_query_selector(&html, &selector)?;
                self.state.stack.push(result);
            }
            OpCode::HtmlQueryAll => {
                let selector = self.get_operand_string(instr, 0)?;
                let html = self.pop_string()?;
                let results = self.parsers.html_query_all(&html, &selector)?;
                self.state.stack.push(Value::List(results));
            }

            // Resource extraction
            OpCode::ExtractScripts => {
                let html = self.pop_string()?;
                let scripts = self.parsers.extract_scripts(&html)?;
                self.state.stack.push(Value::List(scripts));
            }
            OpCode::ExtractStyles => {
                let html = self.pop_string()?;
                let styles = self.parsers.extract_styles(&html)?;
                self.state.stack.push(Value::List(styles));
            }
            OpCode::DetectLibrary => {
                let url = self.pop_string()?;
                let lib = self.parsers.detect_library(&url)?;
                self.state.stack.push(lib);
            }

            // Not implemented yet
            _ => {
                if self.debug {
                    eprintln!("[VM] Unimplemented opcode: {:?}", instr.opcode);
                }
            }
        }

        Ok(())
    }

    // Helper methods

    fn pop(&mut self) -> Result<Value, VMError> {
        self.state.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value, VMError> {
        self.state.stack.last().ok_or(VMError::StackUnderflow)
    }

    fn pop_string(&mut self) -> Result<String, VMError> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            v => Err(VMError::TypeError {
                expected: "string".into(),
                got: format!("{:?}", v),
            }),
        }
    }

    fn pop_int(&mut self) -> Result<i64, VMError> {
        match self.pop()? {
            Value::Int(i) => Ok(i),
            v => Err(VMError::TypeError {
                expected: "int".into(),
                got: format!("{:?}", v),
            }),
        }
    }

    fn get_operand_string(&self, instr: &Instruction, idx: usize) -> Result<String, VMError> {
        instr.operands.get(idx)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .ok_or_else(|| VMError::InvalidOperand(format!("operand {} is not a string", idx)))
    }

    fn get_operand_int(&self, instr: &Instruction, idx: usize) -> Result<i64, VMError> {
        instr.operands.get(idx)
            .and_then(|v| v.as_int())
            .ok_or_else(|| VMError::InvalidOperand(format!("operand {} is not an int", idx)))
    }

    fn resolve_jump_target(&self, instr: &Instruction, program: &Program) -> Result<usize, VMError> {
        if let Some(Value::String(label)) = instr.operands.first() {
            program.labels.get(label)
                .copied()
                .ok_or_else(|| VMError::InvalidOperand(format!("unknown label: {}", label)))
        } else if let Some(Value::Int(offset)) = instr.operands.first() {
            Ok(*offset as usize)
        } else {
            Err(VMError::InvalidOperand("jump target required".into()))
        }
    }

    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, VMError> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| VMError::RegexError(e.to_string()))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
