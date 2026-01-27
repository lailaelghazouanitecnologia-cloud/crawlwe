"""
CrawlWe VM - Virtual Machine Executor

Executes compiled bytecode programs.
"""

import re
from typing import Any, Dict, List, Optional
from dataclasses import dataclass, field
from .opcodes import OpCode, Program, Instruction


@dataclass
class VMState:
    """VM execution state"""
    stack: List[Any] = field(default_factory=list)
    variables: Dict[str, Any] = field(default_factory=dict)
    results: Dict[str, List[Any]] = field(default_factory=dict)
    ip: int = 0  # Instruction pointer
    running: bool = True
    source: str = ""  # Current source being analyzed


class VMError(Exception):
    """VM execution error"""
    pass


class VM:
    """Virtual Machine for executing parser bytecode"""

    def __init__(self, debug=False):
        self.debug = debug
        self.state: Optional[VMState] = None
        self.program: Optional[Program] = None

        # Built-in handlers for domain-specific opcodes
        self.handlers = {
            # Stack operations
            OpCode.NOP: self._nop,
            OpCode.HALT: self._halt,
            OpCode.PUSH: self._push,
            OpCode.POP: self._pop,
            OpCode.DUP: self._dup,
            OpCode.SWAP: self._swap,

            # Load/Store
            OpCode.LOAD_SOURCE: self._load_source,
            OpCode.LOAD_CONST: self._load_const,
            OpCode.LOAD_VAR: self._load_var,
            OpCode.STORE_VAR: self._store_var,
            OpCode.LOAD_RESULT: self._load_result,
            OpCode.STORE_RESULT: self._store_result,

            # Pattern matching
            OpCode.MATCH_REGEX: self._match_regex,
            OpCode.MATCH_ALL: self._match_all,
            OpCode.MATCH_FIRST: self._match_first,
            OpCode.EXTRACT_GROUP: self._extract_group,
            OpCode.TEST_PATTERN: self._test_pattern,

            # String operations
            OpCode.CONCAT: self._concat,
            OpCode.SPLIT: self._split,
            OpCode.TRIM: self._trim,
            OpCode.CONTAINS: self._contains,

            # List/Dict operations
            OpCode.LIST_NEW: self._list_new,
            OpCode.LIST_APPEND: self._list_append,
            OpCode.LIST_GET: self._list_get,
            OpCode.LIST_LEN: self._list_len,
            OpCode.DICT_NEW: self._dict_new,
            OpCode.DICT_SET: self._dict_set,
            OpCode.DICT_GET: self._dict_get,

            # Control flow
            OpCode.JUMP: self._jump,
            OpCode.JUMP_IF: self._jump_if,
            OpCode.JUMP_NOT: self._jump_not,

            # Comparison
            OpCode.EQ: self._eq,
            OpCode.NE: self._ne,
            OpCode.LT: self._lt,
            OpCode.GT: self._gt,
            OpCode.AND: self._and,
            OpCode.OR: self._or,
            OpCode.NOT: self._not,
            OpCode.ADD: self._add,
            OpCode.SUB: self._sub,

            # Emit
            OpCode.EMIT: self._emit,
            OpCode.EMIT_TAGGED: self._emit_tagged,
            OpCode.COLLECT: self._collect,

            # CSS specific
            OpCode.CSS_FIND_KEYFRAMES: self._css_find_keyframes,
            OpCode.CSS_FIND_VARS: self._css_find_vars,

            # HTML specific
            OpCode.HTML_FIND_TAG: self._html_find_tag,
            OpCode.HTML_FIND_CLASS: self._html_find_class,
            OpCode.HTML_FIND_ID: self._html_find_id,
            OpCode.HTML_GET_ATTR: self._html_get_attr,

            # JS specific
            OpCode.JS_FIND_PATTERN: self._js_find_pattern,
            OpCode.JS_FIND_CALL: self._js_find_call,
        }

    def run(self, program: Program, source: str) -> Dict[str, List[Any]]:
        """Execute program on source, return results"""
        self.program = program
        self.state = VMState(source=source)

        if self.debug:
            print(f"=== Running: {program.name} ===")
            print(f"Source length: {len(source)} bytes")
            print(f"Instructions: {len(program.instructions)}")
            print()

        while self.state.running and self.state.ip < len(program.instructions):
            instr = program.instructions[self.state.ip]

            if self.debug:
                self._debug_instruction(instr)

            self._execute(instr)
            self.state.ip += 1

        return self.state.results

    def _execute(self, instr: Instruction):
        """Execute single instruction"""
        handler = self.handlers.get(instr.opcode)
        if handler:
            handler(instr)
        else:
            if self.debug:
                print(f"  [WARN] Unhandled opcode: {instr.opcode.name}")

    def _debug_instruction(self, instr: Instruction):
        """Print debug info for instruction"""
        stack_preview = str(self.state.stack[-3:]) if self.state.stack else "[]"
        print(f"{self.state.ip:04d} {instr.opcode.name:20} | stack: {stack_preview}")

    # ========================================
    # Stack Operations
    # ========================================

    def _nop(self, instr):
        pass

    def _halt(self, instr):
        self.state.running = False

    def _push(self, instr):
        value = instr.operands[0] if instr.operands else None
        self.state.stack.append(value)

    def _pop(self, instr):
        if self.state.stack:
            return self.state.stack.pop()
        return None

    def _dup(self, instr):
        if self.state.stack:
            self.state.stack.append(self.state.stack[-1])

    def _swap(self, instr):
        if len(self.state.stack) >= 2:
            self.state.stack[-1], self.state.stack[-2] = \
                self.state.stack[-2], self.state.stack[-1]

    # ========================================
    # Load/Store Operations
    # ========================================

    def _load_source(self, instr):
        self.state.stack.append(self.state.source)

    def _load_const(self, instr):
        idx = instr.operands[0]
        value = self.program.constants[idx]
        self.state.stack.append(value)

    def _load_var(self, instr):
        name = instr.operands[0]
        value = self.state.variables.get(name)
        self.state.stack.append(value)

    def _store_var(self, instr):
        name = instr.operands[0]
        value = self.state.stack.pop() if self.state.stack else None
        self.state.variables[name] = value

    def _load_result(self, instr):
        self.state.stack.append(self.state.results)

    def _store_result(self, instr):
        if self.state.stack:
            value = self.state.stack.pop()
            if isinstance(value, dict):
                self.state.results.update(value)

    # ========================================
    # Pattern Matching
    # ========================================

    def _match_regex(self, instr):
        pattern = self.state.stack.pop()
        text = self.state.stack.pop()
        try:
            match = re.search(pattern, text)
            self.state.stack.append(bool(match))
        except re.error:
            self.state.stack.append(False)

    def _match_all(self, instr):
        pattern = self.state.stack.pop()
        text = self.state.stack.pop()
        try:
            matches = re.findall(pattern, text)
            # Convert to list of match objects info
            result = []
            for match in re.finditer(pattern, text):
                result.append({
                    'match': match.group(0),
                    'groups': match.groups(),
                    'start': match.start(),
                    'end': match.end()
                })
            self.state.stack.append(result)
        except re.error as e:
            if self.debug:
                print(f"  [ERROR] Regex error: {e}")
            self.state.stack.append([])

    def _match_first(self, instr):
        pattern = self.state.stack.pop()
        text = self.state.stack.pop()
        try:
            match = re.search(pattern, text)
            if match:
                self.state.stack.append({
                    'match': match.group(0),
                    'groups': match.groups(),
                    'start': match.start(),
                    'end': match.end()
                })
            else:
                self.state.stack.append(None)
        except re.error:
            self.state.stack.append(None)

    def _extract_group(self, instr):
        group_idx = instr.operands[0]
        match_info = self.state.stack.pop()
        if match_info and 'groups' in match_info:
            groups = match_info['groups']
            if 0 <= group_idx - 1 < len(groups):
                self.state.stack.append(groups[group_idx - 1])
            else:
                self.state.stack.append(match_info.get('match', ''))
        else:
            self.state.stack.append('')

    def _test_pattern(self, instr):
        pattern = self.state.stack.pop()
        text = self.state.stack.pop()
        try:
            match = re.search(pattern, text)
            self.state.stack.append(bool(match))
        except re.error:
            self.state.stack.append(False)

    # ========================================
    # String Operations
    # ========================================

    def _concat(self, instr):
        b = str(self.state.stack.pop())
        a = str(self.state.stack.pop())
        self.state.stack.append(a + b)

    def _split(self, instr):
        delimiter = self.state.stack.pop()
        text = self.state.stack.pop()
        self.state.stack.append(text.split(delimiter))

    def _trim(self, instr):
        text = self.state.stack.pop()
        self.state.stack.append(text.strip())

    def _contains(self, instr):
        needle = self.state.stack.pop()
        haystack = self.state.stack.pop()
        self.state.stack.append(needle in haystack)

    # ========================================
    # List/Dict Operations
    # ========================================

    def _list_new(self, instr):
        self.state.stack.append([])

    def _list_append(self, instr):
        item = self.state.stack.pop()
        lst = self.state.stack.pop()
        lst.append(item)
        self.state.stack.append(lst)

    def _list_get(self, instr):
        idx = self.state.stack.pop()
        lst = self.state.stack.pop()
        if lst and 0 <= idx < len(lst):
            self.state.stack.append(lst[idx])
        else:
            self.state.stack.append(None)

    def _list_len(self, instr):
        lst = self.state.stack.pop()
        self.state.stack.append(len(lst) if lst else 0)

    def _dict_new(self, instr):
        self.state.stack.append({})

    def _dict_set(self, instr):
        value = self.state.stack.pop()
        key = self.state.stack.pop()
        d = self.state.stack.pop()
        if key not in d:
            d[key] = []
        if isinstance(d[key], list):
            d[key].append(value)
        else:
            d[key] = [d[key], value]
        self.state.stack.append(d)

    def _dict_get(self, instr):
        key = self.state.stack.pop()
        d = self.state.stack.pop()
        self.state.stack.append(d.get(key))

    # ========================================
    # Control Flow
    # ========================================

    def _jump(self, instr):
        target = instr.operands[0]
        if isinstance(target, str):
            # Label reference
            target = self.program.labels.get(target, self.state.ip)
        self.state.ip = target - 1  # -1 because ip will be incremented

    def _jump_if(self, instr):
        condition = self.state.stack.pop()
        if condition:
            self._jump(instr)

    def _jump_not(self, instr):
        condition = self.state.stack.pop()
        if not condition:
            self._jump(instr)

    # ========================================
    # Comparison
    # ========================================

    def _eq(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a == b)

    def _ne(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a != b)

    def _lt(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a < b)

    def _gt(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a > b)

    def _and(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a and b)

    def _or(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        self.state.stack.append(a or b)

    def _not(self, instr):
        a = self.state.stack.pop()
        self.state.stack.append(not a)

    def _add(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        if a is None:
            a = 0
        if b is None:
            b = 0
        self.state.stack.append(a + b)

    def _sub(self, instr):
        b = self.state.stack.pop()
        a = self.state.stack.pop()
        if a is None:
            a = 0
        if b is None:
            b = 0
        self.state.stack.append(a - b)

    # ========================================
    # Emit Operations
    # ========================================

    def _emit(self, instr):
        tag = self.state.stack.pop()
        if tag not in self.state.results:
            self.state.results[tag] = []
        self.state.results[tag].append(True)

    def _emit_tagged(self, instr):
        num_args = instr.operands[0] if instr.operands else 0
        args = []
        for _ in range(num_args):
            args.insert(0, self.state.stack.pop())
        tag = self.state.stack.pop()

        if tag not in self.state.results:
            self.state.results[tag] = []
        self.state.results[tag].append(args if args else True)

    def _collect(self, instr):
        value = self.state.stack.pop()
        tag = instr.operands[0] if instr.operands else "collected"
        if tag not in self.state.results:
            self.state.results[tag] = []
        self.state.results[tag].append(value)

    # ========================================
    # CSS Specific
    # ========================================

    def _css_find_keyframes(self, instr):
        source = self.state.stack.pop()
        pattern = r'@keyframes\s+([a-zA-Z0-9_-]+)\s*\{((?:[^{}]+|\{[^{}]*\})*)\}'
        matches = re.findall(pattern, source, re.DOTALL)
        results = [{'name': m[0], 'body': m[1].strip()} for m in matches]
        self.state.stack.append(results)

    def _css_find_vars(self, instr):
        source = self.state.stack.pop()
        pattern = r'--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);'
        matches = re.findall(pattern, source)
        results = {m[0]: m[1].strip() for m in matches}
        self.state.stack.append(results)

    # ========================================
    # HTML Specific
    # ========================================

    def _html_find_tag(self, instr):
        tag_name = self.state.stack.pop()
        source = self.state.stack.pop()
        pattern = rf'<{tag_name}[^>]*>.*?</{tag_name}>'
        matches = re.findall(pattern, source, re.DOTALL | re.IGNORECASE)
        self.state.stack.append(matches)

    def _html_find_class(self, instr):
        class_name = self.state.stack.pop()
        source = self.state.stack.pop()
        pattern = rf'class=["\'][^"\']*\b{re.escape(class_name)}\b[^"\']*["\']'
        matches = re.findall(pattern, source, re.IGNORECASE)
        self.state.stack.append(matches)

    def _html_find_id(self, instr):
        id_name = self.state.stack.pop()
        source = self.state.stack.pop()
        pattern = rf'id=["\']?{re.escape(id_name)}["\']?'
        matches = re.findall(pattern, source, re.IGNORECASE)
        self.state.stack.append(matches)

    def _html_get_attr(self, instr):
        attr_name = instr.operands[0]
        element = self.state.stack.pop()
        pattern = rf'{attr_name}=["\']([^"\']+)["\']'
        match = re.search(pattern, element)
        self.state.stack.append(match.group(1) if match else None)

    # ========================================
    # JS Specific
    # ========================================

    def _js_find_pattern(self, instr):
        pattern = self.state.stack.pop()
        source = self.state.stack.pop()
        try:
            matches = re.findall(pattern, source)
            self.state.stack.append(matches)
        except re.error:
            self.state.stack.append([])

    def _js_find_call(self, instr):
        func_name = self.state.stack.pop()
        source = self.state.stack.pop()
        pattern = rf'{re.escape(func_name)}\s*\([^)]*\)'
        matches = re.findall(pattern, source)
        self.state.stack.append(matches)
