"""
CrawlWe VM - Opcode definitions

Each opcode is a single byte instruction that the VM executes.
Operands follow the opcode in the bytecode stream.
"""

from enum import IntEnum, auto
from dataclasses import dataclass
from typing import Any, List, Optional


class OpCode(IntEnum):
    """VM Instruction Set"""

    # ========================================
    # STACK OPERATIONS (0x00 - 0x0F)
    # ========================================
    NOP = 0x00          # No operation
    HALT = 0x01         # Stop execution
    PUSH = 0x02         # Push constant onto stack
    POP = 0x03          # Pop top of stack
    DUP = 0x04          # Duplicate top of stack
    SWAP = 0x05         # Swap top two stack items

    # ========================================
    # LOAD/STORE (0x10 - 0x1F)
    # ========================================
    LOAD_SOURCE = 0x10      # Load source text into register
    LOAD_CONST = 0x11       # Load constant from pool
    LOAD_VAR = 0x12         # Load variable by name
    STORE_VAR = 0x13        # Store to variable
    LOAD_RESULT = 0x14      # Load from result accumulator
    STORE_RESULT = 0x15     # Store to result accumulator

    # ========================================
    # PATTERN MATCHING (0x20 - 0x3F)
    # ========================================
    MATCH_REGEX = 0x20      # Match regex pattern, push bool
    MATCH_ALL = 0x21        # Find all matches, push list
    MATCH_FIRST = 0x22      # Find first match, push or null
    EXTRACT_GROUP = 0x23    # Extract capture group by index
    EXTRACT_NAMED = 0x24    # Extract named capture group
    TEST_PATTERN = 0x25     # Test if pattern exists (bool)

    # ========================================
    # STRING OPERATIONS (0x40 - 0x4F)
    # ========================================
    CONCAT = 0x40           # Concatenate strings
    SPLIT = 0x41            # Split string by delimiter
    TRIM = 0x42             # Trim whitespace
    LOWER = 0x43            # To lowercase
    UPPER = 0x44            # To uppercase
    SUBSTR = 0x45           # Substring extraction
    REPLACE = 0x46          # Replace pattern
    CONTAINS = 0x47         # Check if contains substring

    # ========================================
    # LIST/DICT OPERATIONS (0x50 - 0x5F)
    # ========================================
    LIST_NEW = 0x50         # Create new list
    LIST_APPEND = 0x51      # Append to list
    LIST_GET = 0x52         # Get item by index
    LIST_LEN = 0x53         # Get list length
    LIST_ITER = 0x54        # Start iteration
    DICT_NEW = 0x55         # Create new dict
    DICT_SET = 0x56         # Set dict key
    DICT_GET = 0x57         # Get dict value
    DICT_KEYS = 0x58        # Get dict keys

    # ========================================
    # CONTROL FLOW (0x60 - 0x6F)
    # ========================================
    JUMP = 0x60             # Unconditional jump
    JUMP_IF = 0x61          # Jump if top of stack is truthy
    JUMP_NOT = 0x62         # Jump if top of stack is falsy
    LOOP_START = 0x63       # Mark loop start
    LOOP_END = 0x64         # Loop back or exit
    CALL = 0x65             # Call subroutine
    RETURN = 0x66           # Return from subroutine

    # ========================================
    # COMPARISON (0x70 - 0x7F)
    # ========================================
    EQ = 0x70               # Equal
    NE = 0x71               # Not equal
    LT = 0x72               # Less than
    LE = 0x73               # Less or equal
    GT = 0x74               # Greater than
    GE = 0x75               # Greater or equal
    AND = 0x76              # Logical AND
    OR = 0x77               # Logical OR
    NOT = 0x78              # Logical NOT
    ADD = 0x79              # Add two values
    SUB = 0x7A              # Subtract two values
    MUL = 0x7B              # Multiply two values
    DIV = 0x7C              # Divide two values

    # ========================================
    # EMIT/OUTPUT (0x80 - 0x8F)
    # ========================================
    EMIT = 0x80             # Emit token/result
    EMIT_TAGGED = 0x81      # Emit with tag/category
    EMIT_IF = 0x82          # Conditional emit
    COLLECT = 0x83          # Collect into result set

    # ========================================
    # CSS SPECIFIC (0xA0 - 0xAF)
    # ========================================
    CSS_PARSE_SELECTOR = 0xA0   # Parse CSS selector
    CSS_PARSE_PROPERTY = 0xA1   # Parse CSS property
    CSS_PARSE_VALUE = 0xA2      # Parse CSS value
    CSS_PARSE_RULE = 0xA3       # Parse full CSS rule
    CSS_FIND_KEYFRAMES = 0xA4   # Find @keyframes
    CSS_FIND_MEDIA = 0xA5       # Find @media queries
    CSS_FIND_VARS = 0xA6        # Find CSS variables

    # ========================================
    # HTML SPECIFIC (0xB0 - 0xBF)
    # ========================================
    HTML_PARSE = 0xB0           # Parse HTML into DOM
    HTML_FIND_TAG = 0xB1        # Find elements by tag
    HTML_FIND_CLASS = 0xB2      # Find elements by class
    HTML_FIND_ID = 0xB3         # Find element by ID
    HTML_GET_ATTR = 0xB4        # Get attribute value
    HTML_GET_TEXT = 0xB5        # Get text content
    HTML_TRAVERSE = 0xB6        # Traverse DOM tree

    # ========================================
    # JS SPECIFIC (0xC0 - 0xCF)
    # ========================================
    JS_FIND_FUNC = 0xC0         # Find function definitions
    JS_FIND_CLASS = 0xC1        # Find class definitions
    JS_FIND_IMPORT = 0xC2       # Find imports
    JS_FIND_CALL = 0xC3         # Find function calls
    JS_FIND_PATTERN = 0xC4      # Find code pattern
    JS_EXTRACT_STRING = 0xC5    # Extract string literals


@dataclass
class Instruction:
    """Single VM instruction with operands"""
    opcode: OpCode
    operands: List[Any] = None
    line: int = 0  # Source line for debugging

    def __post_init__(self):
        if self.operands is None:
            self.operands = []

    def __repr__(self):
        if self.operands:
            return f"{self.opcode.name} {self.operands}"
        return self.opcode.name


@dataclass
class Program:
    """Compiled bytecode program"""
    instructions: List[Instruction]
    constants: List[Any]  # Constant pool
    labels: dict  # Label -> instruction index
    name: str = "unnamed"

    def __len__(self):
        return len(self.instructions)

    def disassemble(self) -> str:
        """Human-readable disassembly"""
        lines = [f"=== Program: {self.name} ==="]
        lines.append(f"Constants: {self.constants}")
        lines.append(f"Labels: {self.labels}")
        lines.append("-" * 40)

        for i, instr in enumerate(self.instructions):
            label = ""
            for name, idx in self.labels.items():
                if idx == i:
                    label = f"{name}: "
                    break
            lines.append(f"{i:04d} {label}{instr}")

        return "\n".join(lines)
