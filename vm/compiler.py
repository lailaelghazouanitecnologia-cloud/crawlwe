"""
CrawlWe VM - Bytecode Compiler

Compiles DSL AST to VM bytecode.
"""

from typing import List, Dict, Any
from .opcodes import OpCode, Instruction, Program
from .dsl import (
    ProgramNode, SectionNode, RuleNode,
    StatementNode, ActionNode
)


class Compiler:
    """Compiles DSL AST to bytecode"""

    def __init__(self):
        self.instructions: List[Instruction] = []
        self.constants: List[Any] = []
        self.labels: Dict[str, int] = {}
        self.label_counter = 0

    def reset(self):
        """Reset compiler state"""
        self.instructions = []
        self.constants = []
        self.labels = {}
        self.label_counter = 0

    def emit(self, opcode: OpCode, *operands, line=0):
        """Emit an instruction"""
        self.instructions.append(Instruction(
            opcode=opcode,
            operands=list(operands),
            line=line
        ))
        return len(self.instructions) - 1

    def add_constant(self, value: Any) -> int:
        """Add constant to pool, return index"""
        if value in self.constants:
            return self.constants.index(value)
        self.constants.append(value)
        return len(self.constants) - 1

    def make_label(self, prefix="L") -> str:
        """Generate unique label"""
        label = f"{prefix}_{self.label_counter}"
        self.label_counter += 1
        return label

    def mark_label(self, label: str):
        """Mark current position with label"""
        self.labels[label] = len(self.instructions)

    def compile(self, ast: ProgramNode) -> Dict[str, Program]:
        """Compile entire program, returns dict of section programs"""
        programs = {}

        for section in ast.sections:
            self.reset()
            self.compile_section(section)
            self.emit(OpCode.HALT)

            programs[section.type] = Program(
                instructions=self.instructions.copy(),
                constants=self.constants.copy(),
                labels=self.labels.copy(),
                name=f"{section.type}_parser"
            )

        return programs

    def compile_section(self, section: SectionNode):
        """Compile a section (@css, @js, @html)"""
        # Initialize result accumulator
        self.emit(OpCode.DICT_NEW)
        self.emit(OpCode.STORE_VAR, "results")

        for rule in section.rules:
            self.compile_rule(rule, section.type)

        # Load final results
        self.emit(OpCode.LOAD_VAR, "results")

    def compile_rule(self, rule: RuleNode, section_type: str):
        """Compile a rule definition"""
        # Mark rule start for debugging
        rule_label = self.make_label(f"rule_{rule.name}")
        self.mark_label(rule_label)

        for stmt in rule.statements:
            self.compile_statement(stmt, section_type)

    def compile_statement(self, stmt: StatementNode, section_type: str):
        """Compile a statement"""
        if stmt.type == 'match':
            self.compile_match(stmt, section_type)
        elif stmt.type == 'find':
            self.compile_find(stmt, section_type)
        elif stmt.type == 'if':
            self.compile_if(stmt, section_type)
        elif stmt.type == 'foreach':
            self.compile_foreach(stmt, section_type)
        elif stmt.type == 'emit':
            self.compile_emit(stmt.action)
        elif stmt.type == 'set':
            self.compile_set(stmt)

    def compile_match(self, stmt: StatementNode, section_type: str):
        """Compile match statement"""
        # Load source
        self.emit(OpCode.LOAD_SOURCE)

        # Load pattern constant
        pattern_idx = self.add_constant(stmt.pattern)
        self.emit(OpCode.LOAD_CONST, pattern_idx)

        # Execute match
        self.emit(OpCode.MATCH_ALL)

        # Store matches in temp var
        self.emit(OpCode.STORE_VAR, "_matches")

        # Check if any matches
        self.emit(OpCode.LOAD_VAR, "_matches")
        self.emit(OpCode.LIST_LEN)
        self.emit(OpCode.PUSH, 0)
        self.emit(OpCode.GT)

        # Skip action if no matches
        skip_label = self.make_label("skip")
        self.emit(OpCode.JUMP_NOT, skip_label)

        # Iterate over matches
        loop_start = self.make_label("loop_start")
        loop_end = self.make_label("loop_end")

        self.emit(OpCode.PUSH, 0)  # index
        self.emit(OpCode.STORE_VAR, "_i")

        self.mark_label(loop_start)

        # Check loop condition
        self.emit(OpCode.LOAD_VAR, "_i")
        self.emit(OpCode.LOAD_VAR, "_matches")
        self.emit(OpCode.LIST_LEN)
        self.emit(OpCode.LT)
        self.emit(OpCode.JUMP_NOT, loop_end)

        # Get current match
        self.emit(OpCode.LOAD_VAR, "_matches")
        self.emit(OpCode.LOAD_VAR, "_i")
        self.emit(OpCode.LIST_GET)
        self.emit(OpCode.STORE_VAR, "_current")

        # Extract groups and emit
        if stmt.action:
            self.compile_action_with_groups(stmt.action)

        # Increment index
        self.emit(OpCode.LOAD_VAR, "_i")
        self.emit(OpCode.PUSH, 1)
        self.emit(OpCode.ADD)
        self.emit(OpCode.STORE_VAR, "_i")

        self.emit(OpCode.JUMP, loop_start)

        self.mark_label(loop_end)
        self.mark_label(skip_label)

    def compile_find(self, stmt: StatementNode, section_type: str):
        """Compile find statement (HTML specific)"""
        # Parse the selector pattern
        pattern = stmt.pattern

        if section_type == 'html':
            if 'class=' in pattern:
                # find class="modal"
                match = pattern.split('=')
                if len(match) >= 2:
                    class_name = match[1].strip('"\'')
                    self.emit(OpCode.LOAD_SOURCE)
                    class_idx = self.add_constant(class_name)
                    self.emit(OpCode.LOAD_CONST, class_idx)
                    self.emit(OpCode.HTML_FIND_CLASS)

            elif 'tag=' in pattern:
                # find tag="canvas"
                match = pattern.split('=')
                if len(match) >= 2:
                    tag_name = match[1].strip('"\'')
                    self.emit(OpCode.LOAD_SOURCE)
                    tag_idx = self.add_constant(tag_name)
                    self.emit(OpCode.LOAD_CONST, tag_idx)
                    self.emit(OpCode.HTML_FIND_TAG)

            elif 'id=' in pattern:
                # find id="main"
                match = pattern.split('=')
                if len(match) >= 2:
                    id_name = match[1].strip('"\'')
                    self.emit(OpCode.LOAD_SOURCE)
                    id_idx = self.add_constant(id_name)
                    self.emit(OpCode.LOAD_CONST, id_idx)
                    self.emit(OpCode.HTML_FIND_ID)

        # Store results
        self.emit(OpCode.STORE_VAR, "_found")

        # Check if found and emit
        self.emit(OpCode.LOAD_VAR, "_found")
        self.emit(OpCode.LIST_LEN)
        self.emit(OpCode.PUSH, 0)
        self.emit(OpCode.GT)

        skip_label = self.make_label("skip")
        self.emit(OpCode.JUMP_NOT, skip_label)

        if stmt.action:
            self.compile_emit(stmt.action)

        self.mark_label(skip_label)

    def compile_if(self, stmt: StatementNode, section_type: str):
        """Compile if statement"""
        condition = stmt.condition

        # Parse condition
        if 'contains' in condition:
            # if contains "text"
            parts = condition.split('contains')
            if len(parts) >= 2:
                text = parts[1].strip().strip('"\'')
                self.emit(OpCode.LOAD_SOURCE)
                text_idx = self.add_constant(text)
                self.emit(OpCode.LOAD_CONST, text_idx)
                self.emit(OpCode.CONTAINS)
        else:
            # Generic condition - load as test pattern
            self.emit(OpCode.LOAD_SOURCE)
            cond_idx = self.add_constant(condition)
            self.emit(OpCode.LOAD_CONST, cond_idx)
            self.emit(OpCode.TEST_PATTERN)

        skip_label = self.make_label("skip")
        self.emit(OpCode.JUMP_NOT, skip_label)

        # Compile body or action
        if stmt.body:
            for s in stmt.body:
                self.compile_statement(s, section_type)
        elif stmt.action:
            self.compile_emit(stmt.action)

        self.mark_label(skip_label)

    def compile_foreach(self, stmt: StatementNode, section_type: str):
        """Compile foreach statement"""
        # Parse "var in source"
        parts = stmt.pattern.split(' in ')
        if len(parts) != 2:
            return

        var_name = parts[0].strip()
        source_name = parts[1].strip()

        # Load source list
        self.emit(OpCode.LOAD_VAR, source_name)
        self.emit(OpCode.STORE_VAR, "_iter_list")

        # Initialize index
        self.emit(OpCode.PUSH, 0)
        self.emit(OpCode.STORE_VAR, "_iter_i")

        loop_start = self.make_label("foreach_start")
        loop_end = self.make_label("foreach_end")

        self.mark_label(loop_start)

        # Check condition
        self.emit(OpCode.LOAD_VAR, "_iter_i")
        self.emit(OpCode.LOAD_VAR, "_iter_list")
        self.emit(OpCode.LIST_LEN)
        self.emit(OpCode.LT)
        self.emit(OpCode.JUMP_NOT, loop_end)

        # Get current item
        self.emit(OpCode.LOAD_VAR, "_iter_list")
        self.emit(OpCode.LOAD_VAR, "_iter_i")
        self.emit(OpCode.LIST_GET)
        self.emit(OpCode.STORE_VAR, var_name)

        # Compile body
        if stmt.body:
            for s in stmt.body:
                self.compile_statement(s, section_type)

        # Increment
        self.emit(OpCode.LOAD_VAR, "_iter_i")
        self.emit(OpCode.PUSH, 1)
        self.emit(OpCode.ADD)
        self.emit(OpCode.STORE_VAR, "_iter_i")

        self.emit(OpCode.JUMP, loop_start)
        self.mark_label(loop_end)

    def compile_set(self, stmt: StatementNode):
        """Compile set statement"""
        var_name = stmt.pattern
        if stmt.action and stmt.action.args:
            value = stmt.action.args[0]

            # Try to parse as number
            try:
                num_val = int(value)
                self.emit(OpCode.PUSH, num_val)
            except ValueError:
                try:
                    num_val = float(value)
                    self.emit(OpCode.PUSH, num_val)
                except ValueError:
                    # String constant
                    val_idx = self.add_constant(value)
                    self.emit(OpCode.LOAD_CONST, val_idx)

            self.emit(OpCode.STORE_VAR, var_name)

    def compile_emit(self, action: ActionNode):
        """Compile emit action"""
        tag_idx = self.add_constant(action.tag)

        if action.args:
            # Emit with arguments
            self.emit(OpCode.LOAD_CONST, tag_idx)

            # Load arguments
            for arg in action.args:
                if arg.startswith('$'):
                    # Variable reference
                    if arg[1:].isdigit():
                        # Capture group
                        self.emit(OpCode.LOAD_VAR, "_current")
                        group_num = int(arg[1:])
                        self.emit(OpCode.EXTRACT_GROUP, group_num)
                    else:
                        # Named variable
                        self.emit(OpCode.LOAD_VAR, arg)
                else:
                    # String literal
                    arg_idx = self.add_constant(arg)
                    self.emit(OpCode.LOAD_CONST, arg_idx)

            self.emit(OpCode.EMIT_TAGGED, len(action.args))
        else:
            # Simple emit
            self.emit(OpCode.LOAD_CONST, tag_idx)
            self.emit(OpCode.EMIT)

        # Collect into results
        self.emit(OpCode.LOAD_VAR, "results")
        self.emit(OpCode.LOAD_CONST, tag_idx)
        self.emit(OpCode.LOAD_RESULT)
        self.emit(OpCode.DICT_SET)
        self.emit(OpCode.STORE_VAR, "results")

    def compile_action_with_groups(self, action: ActionNode):
        """Compile action that uses capture groups"""
        self.compile_emit(action)


def compile_dsl(source: str) -> Dict[str, Program]:
    """Compile DSL source to bytecode programs"""
    from .dsl import parse_rules

    ast = parse_rules(source)
    compiler = Compiler()
    return compiler.compile(ast)
