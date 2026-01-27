//! VM Module - Virtual Machine for parsing operations
//!
//! A bytecode VM with pluggable parser backends, similar to Bun's approach.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Compiler                          │
//! │  (DSL rules → Bytecode)                             │
//! └──────────────────────┬──────────────────────────────┘
//!                        │
//!                        ▼
//! ┌─────────────────────────────────────────────────────┐
//! │                      VM                              │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
//! │  │  Stack   │  │ Variables│  │ Results  │          │
//! │  └──────────┘  └──────────┘  └──────────┘          │
//! └──────────────────────┬──────────────────────────────┘
//!                        │
//!                        ▼
//! ┌─────────────────────────────────────────────────────┐
//! │              Parser Registry                         │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
//! │  │  Regex  │ │TreeSitter│ │Html5Ever│ │ Custom  │   │
//! │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crawlwe::vm::{VM, Compiler, ParserBuilder};
//!
//! // Use pre-built analyzer
//! let program = ParserBuilder::css_analyzer();
//! let mut vm = VM::new();
//! let state = vm.execute(&program, css_source)?;
//!
//! // Or build custom program
//! let mut c = Compiler::new("custom");
//! c.load_source();
//! c.match_all(r"#[0-9a-f]{6}");
//! c.collect("colors");
//! let program = c.finish();
//! ```

pub mod opcodes;
pub mod vm;
pub mod parser_registry;
pub mod compiler;

pub use opcodes::{OpCode, Instruction, Value, Program};
pub use vm::{VM, VMState, VMError};
pub use parser_registry::{ParserRegistry, ParserKind, ParserBackend};
pub use compiler::{Compiler, ParserBuilder};
