//! VM Opcodes - Instruction set for the CrawlWe parser VM
//!
//! Similar to Bun's approach: specialized opcodes for different parsing tasks
//! with a unified execution model.

use std::fmt;

/// Opcode categories for fast dispatch
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // ========================================
    // STACK OPERATIONS (0x00 - 0x0F)
    // ========================================
    Nop = 0x00,
    Halt = 0x01,
    Push = 0x02,
    Pop = 0x03,
    Dup = 0x04,
    Swap = 0x05,
    Rot = 0x06,

    // ========================================
    // LOAD/STORE (0x10 - 0x1F)
    // ========================================
    LoadSource = 0x10,
    LoadConst = 0x11,
    LoadVar = 0x12,
    StoreVar = 0x13,
    LoadResult = 0x14,
    StoreResult = 0x15,
    LoadField = 0x16,
    StoreField = 0x17,

    // ========================================
    // PATTERN MATCHING (0x20 - 0x3F)
    // ========================================
    MatchRegex = 0x20,
    MatchAll = 0x21,
    MatchFirst = 0x22,
    ExtractGroup = 0x23,
    ExtractNamed = 0x24,
    TestPattern = 0x25,
    MatchSelector = 0x26,
    MatchXPath = 0x27,

    // ========================================
    // STRING OPERATIONS (0x40 - 0x4F)
    // ========================================
    Concat = 0x40,
    Split = 0x41,
    Trim = 0x42,
    Lower = 0x43,
    Upper = 0x44,
    Substr = 0x45,
    Replace = 0x46,
    Contains = 0x47,
    StartsWith = 0x48,
    EndsWith = 0x49,

    // ========================================
    // COLLECTION OPERATIONS (0x50 - 0x5F)
    // ========================================
    ListNew = 0x50,
    ListAppend = 0x51,
    ListGet = 0x52,
    ListLen = 0x53,
    ListIter = 0x54,
    MapNew = 0x55,
    MapSet = 0x56,
    MapGet = 0x57,
    MapKeys = 0x58,
    MapValues = 0x59,

    // ========================================
    // CONTROL FLOW (0x60 - 0x6F)
    // ========================================
    Jump = 0x60,
    JumpIf = 0x61,
    JumpNot = 0x62,
    LoopStart = 0x63,
    LoopEnd = 0x64,
    Call = 0x65,
    Return = 0x66,
    Yield = 0x67,

    // ========================================
    // COMPARISON & ARITHMETIC (0x70 - 0x7F)
    // ========================================
    Eq = 0x70,
    Ne = 0x71,
    Lt = 0x72,
    Le = 0x73,
    Gt = 0x74,
    Ge = 0x75,
    And = 0x76,
    Or = 0x77,
    Not = 0x78,
    Add = 0x79,
    Sub = 0x7A,
    Mul = 0x7B,
    Div = 0x7C,
    Mod = 0x7D,

    // ========================================
    // EMIT/OUTPUT (0x80 - 0x8F)
    // ========================================
    Emit = 0x80,
    EmitTagged = 0x81,
    EmitIf = 0x82,
    Collect = 0x83,
    EmitError = 0x84,
    EmitWarning = 0x85,

    // ========================================
    // PARSER DISPATCH (0x90 - 0x9F)
    // New: Switch between parser backends
    // ========================================
    UseParser = 0x90,
    ParseWith = 0x91,
    ParserPush = 0x92,
    ParserPop = 0x93,

    // ========================================
    // CSS SPECIFIC (0xA0 - 0xAF)
    // ========================================
    CssParseSelector = 0xA0,
    CssParseProperty = 0xA1,
    CssParseValue = 0xA2,
    CssParseRule = 0xA3,
    CssFindKeyframes = 0xA4,
    CssFindMedia = 0xA5,
    CssFindVars = 0xA6,
    CssFindColors = 0xA7,
    CssFindFonts = 0xA8,

    // ========================================
    // HTML SPECIFIC (0xB0 - 0xBF)
    // ========================================
    HtmlParse = 0xB0,
    HtmlFindTag = 0xB1,
    HtmlFindClass = 0xB2,
    HtmlFindId = 0xB3,
    HtmlGetAttr = 0xB4,
    HtmlGetText = 0xB5,
    HtmlTraverse = 0xB6,
    HtmlQuerySelector = 0xB7,
    HtmlQueryAll = 0xB8,

    // ========================================
    // JS SPECIFIC (0xC0 - 0xCF)
    // ========================================
    JsFindFunc = 0xC0,
    JsFindClass = 0xC1,
    JsFindImport = 0xC2,
    JsFindCall = 0xC3,
    JsFindPattern = 0xC4,
    JsExtractString = 0xC5,
    JsFindExport = 0xC6,
    JsFindVariable = 0xC7,

    // ========================================
    // RESOURCE EXTRACTION (0xD0 - 0xDF)
    // ========================================
    ExtractScripts = 0xD0,
    ExtractStyles = 0xD1,
    ExtractLinks = 0xD2,
    ExtractImages = 0xD3,
    ExtractFonts = 0xD4,
    DetectLibrary = 0xD5,
    ExtractMeta = 0xD6,

    // ========================================
    // ADVANCED (0xE0 - 0xEF)
    // ========================================
    Parallel = 0xE0,
    Async = 0xE1,
    Cache = 0xE2,
    Memoize = 0xE3,
    Profile = 0xE4,
}

impl OpCode {
    /// Get the number of operands this opcode expects
    pub fn operand_count(&self) -> usize {
        match self {
            // No operands
            OpCode::Nop | OpCode::Halt | OpCode::Pop | OpCode::Dup |
            OpCode::Swap | OpCode::Rot | OpCode::Return | OpCode::Yield |
            OpCode::ListNew | OpCode::MapNew | OpCode::Not => 0,

            // One operand
            OpCode::Push | OpCode::LoadConst | OpCode::LoadVar |
            OpCode::StoreVar | OpCode::LoadResult | OpCode::StoreResult |
            OpCode::Jump | OpCode::JumpIf | OpCode::JumpNot |
            OpCode::Call | OpCode::UseParser | OpCode::ListGet => 1,

            // Two operands
            OpCode::LoadField | OpCode::StoreField | OpCode::Substr |
            OpCode::MapSet | OpCode::MapGet => 2,

            // Variable operands (determined at runtime)
            _ => 1,
        }
    }

    /// Check if this opcode can branch
    pub fn is_branch(&self) -> bool {
        matches!(self,
            OpCode::Jump | OpCode::JumpIf | OpCode::JumpNot |
            OpCode::LoopStart | OpCode::LoopEnd | OpCode::Call | OpCode::Return
        )
    }

    /// Check if this opcode is a parser operation
    pub fn is_parser_op(&self) -> bool {
        let code = *self as u8;
        (0x90..=0xDF).contains(&code)
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A single VM instruction
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operands: Vec<Value>,
    pub line: u32,
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operands: Vec::new(),
            line: 0,
        }
    }

    pub fn with_operand(mut self, value: Value) -> Self {
        self.operands.push(value);
        self
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line = line;
        self
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.operands.is_empty() {
            write!(f, "{}", self.opcode)
        } else {
            write!(f, "{} {:?}", self.opcode, self.operands)
        }
    }
}

/// VM Value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Regex(String),
    Selector(String),
    Reference(usize),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Regex(_) => true,
            Value::Selector(_) => true,
            Value::Reference(_) => true,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
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

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

/// Compiled program
#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub labels: std::collections::HashMap<String, usize>,
}

impl Program {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instructions: Vec::new(),
            constants: Vec::new(),
            labels: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, instr: Instruction) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(instr);
        idx
    }

    pub fn add_const(&mut self, value: Value) -> usize {
        // Check if constant already exists
        for (i, v) in self.constants.iter().enumerate() {
            if v == &value {
                return i;
            }
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }

    pub fn label(&mut self, name: impl Into<String>) {
        self.labels.insert(name.into(), self.instructions.len());
    }

    /// Disassemble program to human-readable format
    pub fn disassemble(&self) -> String {
        let mut out = format!("=== Program: {} ===\n", self.name);
        out.push_str(&format!("Constants: {:?}\n", self.constants));
        out.push_str(&format!("Labels: {:?}\n", self.labels));
        out.push_str(&"-".repeat(40));
        out.push('\n');

        for (i, instr) in self.instructions.iter().enumerate() {
            // Check for labels
            for (name, &idx) in &self.labels {
                if idx == i {
                    out.push_str(&format!("{}:\n", name));
                }
            }
            out.push_str(&format!("{:04} {}\n", i, instr));
        }

        out
    }
}
