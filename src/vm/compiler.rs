//! VM Compiler - Compiles DSL rules to bytecode
//!
//! Takes parser rules and compiles them to VM bytecode for fast execution.

use super::opcodes::{OpCode, Instruction, Value, Program};

/// Compiler for parser rules
pub struct Compiler {
    program: Program,
    loop_stack: Vec<(String, String)>,  // (start_label, end_label)
    label_counter: usize,
}

impl Compiler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            program: Program::new(name),
            loop_stack: Vec::new(),
            label_counter: 0,
        }
    }

    /// Generate a unique label
    fn gen_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Emit an instruction
    pub fn emit(&mut self, opcode: OpCode) -> &mut Self {
        self.program.add(Instruction::new(opcode));
        self
    }

    /// Emit instruction with string operand
    pub fn emit_str(&mut self, opcode: OpCode, s: impl Into<String>) -> &mut Self {
        self.program.add(
            Instruction::new(opcode).with_operand(Value::String(s.into()))
        );
        self
    }

    /// Emit instruction with int operand
    pub fn emit_int(&mut self, opcode: OpCode, i: i64) -> &mut Self {
        self.program.add(
            Instruction::new(opcode).with_operand(Value::Int(i))
        );
        self
    }

    /// Add a label at current position
    pub fn label(&mut self, name: impl Into<String>) -> &mut Self {
        self.program.label(name);
        self
    }

    /// Push constant value
    pub fn push_const(&mut self, value: Value) -> &mut Self {
        let idx = self.program.add_const(value);
        self.emit_int(OpCode::LoadConst, idx as i64)
    }

    /// Push string constant
    pub fn push_string(&mut self, s: impl Into<String>) -> &mut Self {
        self.push_const(Value::String(s.into()))
    }

    /// Push int constant
    pub fn push_int(&mut self, i: i64) -> &mut Self {
        self.push_const(Value::Int(i))
    }

    /// Load variable
    pub fn load_var(&mut self, name: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::LoadVar, name)
    }

    /// Store variable
    pub fn store_var(&mut self, name: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::StoreVar, name)
    }

    /// Load source text
    pub fn load_source(&mut self) -> &mut Self {
        self.emit(OpCode::LoadSource)
    }

    /// Match regex and push result
    pub fn match_regex(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::MatchRegex, pattern)
    }

    /// Find all regex matches
    pub fn match_all(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::MatchAll, pattern)
    }

    /// Test if pattern exists
    pub fn test_pattern(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::TestPattern, pattern)
    }

    /// If-then-else
    pub fn if_then<F, G>(&mut self, then_fn: F, else_fn: Option<G>) -> &mut Self
    where
        F: FnOnce(&mut Self),
        G: FnOnce(&mut Self),
    {
        let else_label = self.gen_label("else");
        let end_label = self.gen_label("endif");

        // Jump to else if false
        self.emit_str(OpCode::JumpNot, &else_label);

        // Then block
        then_fn(self);

        // Jump over else
        if else_fn.is_some() {
            self.emit_str(OpCode::Jump, &end_label);
        }

        // Else block
        self.label(&else_label);
        if let Some(else_fn) = else_fn {
            else_fn(self);
        }

        self.label(&end_label);
        self
    }

    /// For-each loop over list on stack
    pub fn foreach<F>(&mut self, body_fn: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let start_label = self.gen_label("loop_start");
        let end_label = self.gen_label("loop_end");

        // Initialize: list is on stack, push index 0
        self.push_int(0);
        self.store_var("__idx");

        // Start label
        self.label(&start_label);

        // Check if index < length
        self.load_var("__idx");
        self.emit(OpCode::Dup);
        self.emit(OpCode::ListLen);
        self.emit(OpCode::Lt);
        self.emit_str(OpCode::JumpNot, &end_label);

        // Get current item
        self.load_var("__idx");
        self.emit_int(OpCode::ListGet, 0);

        // Body
        self.loop_stack.push((start_label.clone(), end_label.clone()));
        body_fn(self);
        self.loop_stack.pop();

        // Increment index
        self.load_var("__idx");
        self.push_int(1);
        self.emit(OpCode::Add);
        self.store_var("__idx");

        // Loop back
        self.emit_str(OpCode::Jump, &start_label);

        // End
        self.label(&end_label);
        self
    }

    /// Emit result
    pub fn emit_result(&mut self) -> &mut Self {
        self.emit(OpCode::Emit)
    }

    /// Emit tagged result
    pub fn emit_tagged(&mut self, tag: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::EmitTagged, tag)
    }

    /// Collect into named result
    pub fn collect(&mut self, name: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::Collect, name)
    }

    /// Use specific parser
    pub fn use_parser(&mut self, parser: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::UseParser, parser)
    }

    /// CSS: Find keyframes
    pub fn css_find_keyframes(&mut self) -> &mut Self {
        self.emit(OpCode::CssFindKeyframes)
    }

    /// CSS: Find variables
    pub fn css_find_vars(&mut self) -> &mut Self {
        self.emit(OpCode::CssFindVars)
    }

    /// CSS: Find colors
    pub fn css_find_colors(&mut self) -> &mut Self {
        self.emit(OpCode::CssFindColors)
    }

    /// HTML: Query selector
    pub fn html_query(&mut self, selector: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::HtmlQuerySelector, selector)
    }

    /// HTML: Query all
    pub fn html_query_all(&mut self, selector: impl Into<String>) -> &mut Self {
        self.emit_str(OpCode::HtmlQueryAll, selector)
    }

    /// Extract scripts from HTML
    pub fn extract_scripts(&mut self) -> &mut Self {
        self.emit(OpCode::ExtractScripts)
    }

    /// Extract styles from HTML
    pub fn extract_styles(&mut self) -> &mut Self {
        self.emit(OpCode::ExtractStyles)
    }

    /// Detect library from URL
    pub fn detect_library(&mut self) -> &mut Self {
        self.emit(OpCode::DetectLibrary)
    }

    /// Halt execution
    pub fn halt(&mut self) -> &mut Self {
        self.emit(OpCode::Halt)
    }

    /// Finish compilation and return program
    pub fn finish(mut self) -> Program {
        // Ensure program ends with halt
        if self.program.instructions.last()
            .map(|i| i.opcode != OpCode::Halt)
            .unwrap_or(true)
        {
            self.emit(OpCode::Halt);
        }
        self.program
    }
}

/// Builder pattern for common parsing tasks
pub struct ParserBuilder;

impl ParserBuilder {
    /// Build a CSS analyzer program
    pub fn css_analyzer() -> Program {
        let mut c = Compiler::new("css_analyzer");

        c.load_source();
        c.emit(OpCode::Dup);
        c.css_find_keyframes();
        c.collect("keyframes");

        c.load_source();
        c.css_find_vars();
        c.collect("variables");

        c.load_source();
        c.css_find_colors();
        c.collect("colors");

        c.finish()
    }

    /// Build an HTML resource extractor program
    pub fn html_extractor() -> Program {
        let mut c = Compiler::new("html_extractor");

        // Extract scripts
        c.load_source();
        c.extract_scripts();
        c.collect("scripts");

        // Extract styles
        c.load_source();
        c.extract_styles();
        c.collect("styles");

        c.finish()
    }

    /// Build a library detector program
    pub fn library_detector() -> Program {
        let mut c = Compiler::new("library_detector");

        // Input: list of URLs on stack
        c.load_source();

        // Parse JSON array of URLs (simplified)
        c.emit_str(OpCode::Split, "\n");

        // For each URL, detect library
        c.foreach(|c| {
            c.emit(OpCode::Trim);
            c.detect_library();
            c.collect("libraries");
        });

        c.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::vm::VM;

    #[test]
    fn test_simple_program() {
        let mut c = Compiler::new("test");
        c.push_string("hello");
        c.emit_result();
        let program = c.finish();

        let mut vm = VM::new();
        let state = vm.execute(&program, "").unwrap();

        assert_eq!(state.results.get("default").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_css_analyzer() {
        let program = ParserBuilder::css_analyzer();
        let css = r#"
            @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
            :root { --primary: #ff0000; }
        "#;

        let mut vm = VM::new();
        let state = vm.execute(&program, css).unwrap();

        assert!(state.results.contains_key("keyframes"));
        assert!(state.results.contains_key("variables"));
        assert!(state.results.contains_key("colors"));
    }
}
