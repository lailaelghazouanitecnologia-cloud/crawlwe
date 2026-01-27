"""
CrawlWe DSL Parser

Mini language for defining parsing rules.

Syntax:
    @css {
        rule "name" {
            match /@keyframes\s+(\w+)/ -> emit ANIMATION($1)
            match /transition:\s*([^;]+)/ -> emit TRANSITION($1)
        }
    }

    @js {
        rule "three_js" {
            match /THREE\.(\w+)/ -> emit THREEJS($1)
            if contains "WebGLRenderer" -> emit WEBGL_DETECTED
        }
    }

    @html {
        rule "components" {
            find class="modal" -> emit UI_MODAL
            find tag="canvas" -> emit CANVAS_ELEMENT
        }
    }
"""

import re
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional
from enum import Enum, auto


class TokenType(Enum):
    """DSL Token types"""
    # Keywords
    RULE = auto()
    MATCH = auto()
    FIND = auto()
    IF = auto()
    ELSE = auto()
    EMIT = auto()
    COLLECT = auto()
    CONTAINS = auto()
    COUNT = auto()
    FOREACH = auto()
    SET = auto()

    # Sections
    CSS = auto()
    JS = auto()
    HTML = auto()

    # Symbols
    LBRACE = auto()
    RBRACE = auto()
    LPAREN = auto()
    RPAREN = auto()
    ARROW = auto()
    COMMA = auto()
    DOT = auto()
    EQUALS = auto()
    AT = auto()

    # Literals
    STRING = auto()
    REGEX = auto()
    NUMBER = auto()
    IDENTIFIER = auto()
    VARIABLE = auto()  # $1, $name, etc.

    # Special
    NEWLINE = auto()
    EOF = auto()


@dataclass
class Token:
    type: TokenType
    value: Any
    line: int
    column: int


@dataclass
class RuleNode:
    """AST node for a rule definition"""
    name: str
    statements: List['StatementNode']


@dataclass
class StatementNode:
    """AST node for a statement"""
    type: str  # 'match', 'find', 'if', 'foreach', 'emit', 'set'
    pattern: Optional[str] = None
    condition: Optional[str] = None
    action: Optional['ActionNode'] = None
    body: Optional[List['StatementNode']] = None


@dataclass
class ActionNode:
    """AST node for an action"""
    type: str  # 'emit', 'collect', 'set'
    tag: str = ""
    args: List[str] = field(default_factory=list)


@dataclass
class SectionNode:
    """AST node for a section (@css, @js, @html)"""
    type: str  # 'css', 'js', 'html'
    rules: List[RuleNode]


@dataclass
class ProgramNode:
    """Root AST node"""
    sections: List[SectionNode]


class Lexer:
    """DSL Lexer/Tokenizer"""

    KEYWORDS = {
        'rule': TokenType.RULE,
        'match': TokenType.MATCH,
        'find': TokenType.FIND,
        'if': TokenType.IF,
        'else': TokenType.ELSE,
        'emit': TokenType.EMIT,
        'collect': TokenType.COLLECT,
        'contains': TokenType.CONTAINS,
        'count': TokenType.COUNT,
        'foreach': TokenType.FOREACH,
        'set': TokenType.SET,
    }

    SECTIONS = {
        'css': TokenType.CSS,
        'js': TokenType.JS,
        'html': TokenType.HTML,
    }

    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens = []

    def peek(self, offset=0) -> str:
        pos = self.pos + offset
        if pos >= len(self.source):
            return '\0'
        return self.source[pos]

    def advance(self) -> str:
        ch = self.peek()
        self.pos += 1
        if ch == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return ch

    def skip_whitespace(self):
        while self.peek() in ' \t\r':
            self.advance()

    def skip_comment(self):
        if self.peek() == '#':
            while self.peek() not in '\n\0':
                self.advance()

    def read_string(self) -> str:
        quote = self.advance()  # consume opening quote
        result = []
        while self.peek() != quote and self.peek() != '\0':
            if self.peek() == '\\':
                self.advance()
                ch = self.advance()
                if ch == 'n':
                    result.append('\n')
                elif ch == 't':
                    result.append('\t')
                else:
                    result.append(ch)
            else:
                result.append(self.advance())
        self.advance()  # consume closing quote
        return ''.join(result)

    def read_regex(self) -> str:
        self.advance()  # consume opening /
        result = []
        while self.peek() != '/' and self.peek() != '\0':
            if self.peek() == '\\':
                result.append(self.advance())
                result.append(self.advance())
            else:
                result.append(self.advance())
        self.advance()  # consume closing /

        # Read flags
        flags = []
        while self.peek() in 'gimsux':
            flags.append(self.advance())

        return ''.join(result)

    def read_identifier(self) -> str:
        result = []
        while self.peek().isalnum() or self.peek() == '_':
            result.append(self.advance())
        return ''.join(result)

    def read_number(self) -> int:
        result = []
        while self.peek().isdigit():
            result.append(self.advance())
        return int(''.join(result))

    def read_variable(self) -> str:
        self.advance()  # consume $
        if self.peek().isdigit():
            return '$' + str(self.read_number())
        else:
            return '$' + self.read_identifier()

    def tokenize(self) -> List[Token]:
        while self.pos < len(self.source):
            self.skip_whitespace()
            self.skip_comment()

            if self.pos >= len(self.source):
                break

            ch = self.peek()
            line, col = self.line, self.column

            # Newline
            if ch == '\n':
                self.advance()
                self.tokens.append(Token(TokenType.NEWLINE, '\n', line, col))
                continue

            # Symbols
            if ch == '{':
                self.advance()
                self.tokens.append(Token(TokenType.LBRACE, '{', line, col))
            elif ch == '}':
                self.advance()
                self.tokens.append(Token(TokenType.RBRACE, '}', line, col))
            elif ch == '(':
                self.advance()
                self.tokens.append(Token(TokenType.LPAREN, '(', line, col))
            elif ch == ')':
                self.advance()
                self.tokens.append(Token(TokenType.RPAREN, ')', line, col))
            elif ch == ',':
                self.advance()
                self.tokens.append(Token(TokenType.COMMA, ',', line, col))
            elif ch == '.':
                self.advance()
                self.tokens.append(Token(TokenType.DOT, '.', line, col))
            elif ch == '=':
                self.advance()
                self.tokens.append(Token(TokenType.EQUALS, '=', line, col))
            elif ch == '@':
                self.advance()
                self.tokens.append(Token(TokenType.AT, '@', line, col))
            elif ch == '-' and self.peek(1) == '>':
                self.advance()
                self.advance()
                self.tokens.append(Token(TokenType.ARROW, '->', line, col))

            # String
            elif ch in '"\'':
                value = self.read_string()
                self.tokens.append(Token(TokenType.STRING, value, line, col))

            # Regex
            elif ch == '/':
                value = self.read_regex()
                self.tokens.append(Token(TokenType.REGEX, value, line, col))

            # Variable
            elif ch == '$':
                value = self.read_variable()
                self.tokens.append(Token(TokenType.VARIABLE, value, line, col))

            # Number
            elif ch.isdigit():
                value = self.read_number()
                self.tokens.append(Token(TokenType.NUMBER, value, line, col))

            # Identifier/Keyword
            elif ch.isalpha() or ch == '_':
                value = self.read_identifier()
                if value in self.KEYWORDS:
                    self.tokens.append(Token(self.KEYWORDS[value], value, line, col))
                elif value in self.SECTIONS:
                    self.tokens.append(Token(self.SECTIONS[value], value, line, col))
                else:
                    self.tokens.append(Token(TokenType.IDENTIFIER, value, line, col))

            elif ch == '\0':
                break

            else:
                raise SyntaxError(f"Unexpected character '{ch}' at line {line}, column {col}")

        self.tokens.append(Token(TokenType.EOF, None, self.line, self.column))
        return self.tokens


class Parser:
    """DSL Parser - converts tokens to AST"""

    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.pos = 0

    def peek(self, offset=0) -> Token:
        pos = self.pos + offset
        if pos >= len(self.tokens):
            return self.tokens[-1]  # EOF
        return self.tokens[pos]

    def advance(self) -> Token:
        token = self.peek()
        self.pos += 1
        return token

    def expect(self, *types: TokenType) -> Token:
        token = self.peek()
        if token.type not in types:
            expected = ' or '.join(t.name for t in types)
            raise SyntaxError(
                f"Expected {expected}, got {token.type.name} at line {token.line}"
            )
        return self.advance()

    def skip_newlines(self):
        while self.peek().type == TokenType.NEWLINE:
            self.advance()

    def parse(self) -> ProgramNode:
        """Parse entire program"""
        sections = []

        self.skip_newlines()
        while self.peek().type != TokenType.EOF:
            if self.peek().type == TokenType.AT:
                section = self.parse_section()
                sections.append(section)
            else:
                self.advance()  # Skip unexpected tokens
            self.skip_newlines()

        return ProgramNode(sections=sections)

    def parse_section(self) -> SectionNode:
        """Parse @css, @js, or @html section"""
        self.expect(TokenType.AT)
        section_type = self.expect(TokenType.CSS, TokenType.JS, TokenType.HTML)
        self.skip_newlines()
        self.expect(TokenType.LBRACE)
        self.skip_newlines()

        rules = []
        while self.peek().type != TokenType.RBRACE:
            if self.peek().type == TokenType.RULE:
                rule = self.parse_rule()
                rules.append(rule)
            elif self.peek().type == TokenType.MATCH:
                # Anonymous rule
                stmt = self.parse_match_statement()
                rules.append(RuleNode(name="_anon", statements=[stmt]))
            self.skip_newlines()

        self.expect(TokenType.RBRACE)

        return SectionNode(type=section_type.value, rules=rules)

    def parse_rule(self) -> RuleNode:
        """Parse rule definition"""
        self.expect(TokenType.RULE)
        name = self.expect(TokenType.STRING).value
        self.skip_newlines()
        self.expect(TokenType.LBRACE)
        self.skip_newlines()

        statements = []
        while self.peek().type != TokenType.RBRACE:
            stmt = self.parse_statement()
            if stmt:
                statements.append(stmt)
            self.skip_newlines()

        self.expect(TokenType.RBRACE)

        return RuleNode(name=name, statements=statements)

    def parse_statement(self) -> Optional[StatementNode]:
        """Parse a single statement"""
        token = self.peek()

        if token.type == TokenType.MATCH:
            return self.parse_match_statement()
        elif token.type == TokenType.FIND:
            return self.parse_find_statement()
        elif token.type == TokenType.IF:
            return self.parse_if_statement()
        elif token.type == TokenType.FOREACH:
            return self.parse_foreach_statement()
        elif token.type == TokenType.EMIT:
            return self.parse_emit_statement()
        elif token.type == TokenType.SET:
            return self.parse_set_statement()
        elif token.type == TokenType.NEWLINE:
            self.advance()
            return None
        else:
            return None

    def parse_match_statement(self) -> StatementNode:
        """Parse: match /pattern/ -> action"""
        self.expect(TokenType.MATCH)
        pattern = self.expect(TokenType.REGEX).value
        self.expect(TokenType.ARROW)
        action = self.parse_action()

        return StatementNode(type='match', pattern=pattern, action=action)

    def parse_find_statement(self) -> StatementNode:
        """Parse: find tag="div" -> action"""
        self.expect(TokenType.FIND)

        # Parse selector
        selector_parts = []
        while self.peek().type not in (TokenType.ARROW, TokenType.NEWLINE, TokenType.EOF):
            token = self.advance()
            selector_parts.append(str(token.value))

        pattern = ' '.join(selector_parts)
        self.expect(TokenType.ARROW)
        action = self.parse_action()

        return StatementNode(type='find', pattern=pattern, action=action)

    def parse_if_statement(self) -> StatementNode:
        """Parse: if condition -> action"""
        self.expect(TokenType.IF)

        # Parse condition
        condition_parts = []
        while self.peek().type not in (TokenType.ARROW, TokenType.LBRACE, TokenType.NEWLINE):
            token = self.advance()
            condition_parts.append(str(token.value))

        condition = ' '.join(condition_parts)

        if self.peek().type == TokenType.ARROW:
            self.advance()
            action = self.parse_action()
            return StatementNode(type='if', condition=condition, action=action)
        else:
            # Block form
            self.skip_newlines()
            self.expect(TokenType.LBRACE)
            self.skip_newlines()
            body = []
            while self.peek().type != TokenType.RBRACE:
                stmt = self.parse_statement()
                if stmt:
                    body.append(stmt)
                self.skip_newlines()
            self.expect(TokenType.RBRACE)
            return StatementNode(type='if', condition=condition, body=body)

    def parse_foreach_statement(self) -> StatementNode:
        """Parse: foreach $item in $list { ... }"""
        self.expect(TokenType.FOREACH)
        var = self.expect(TokenType.VARIABLE).value

        # Expect 'in'
        in_token = self.advance()
        if in_token.value != 'in':
            raise SyntaxError(f"Expected 'in' in foreach, got {in_token.value}")

        source = self.expect(TokenType.VARIABLE).value
        self.skip_newlines()
        self.expect(TokenType.LBRACE)
        self.skip_newlines()

        body = []
        while self.peek().type != TokenType.RBRACE:
            stmt = self.parse_statement()
            if stmt:
                body.append(stmt)
            self.skip_newlines()

        self.expect(TokenType.RBRACE)

        return StatementNode(
            type='foreach',
            pattern=f"{var} in {source}",
            body=body
        )

    def parse_emit_statement(self) -> StatementNode:
        """Parse standalone emit"""
        action = self.parse_action()
        return StatementNode(type='emit', action=action)

    def parse_set_statement(self) -> StatementNode:
        """Parse: set $var = value"""
        self.expect(TokenType.SET)
        var = self.expect(TokenType.VARIABLE).value
        self.expect(TokenType.EQUALS)

        # Parse value
        value_parts = []
        while self.peek().type not in (TokenType.NEWLINE, TokenType.EOF, TokenType.RBRACE):
            token = self.advance()
            value_parts.append(str(token.value))

        value = ' '.join(value_parts)

        return StatementNode(
            type='set',
            pattern=var,
            action=ActionNode(type='set', tag=var, args=[value])
        )

    def parse_action(self) -> ActionNode:
        """Parse action: emit TAG($1, $2) or collect TAG"""
        action_type = self.expect(TokenType.EMIT, TokenType.COLLECT).value

        tag = self.expect(TokenType.IDENTIFIER).value
        args = []

        if self.peek().type == TokenType.LPAREN:
            self.advance()
            while self.peek().type != TokenType.RPAREN:
                if self.peek().type == TokenType.VARIABLE:
                    args.append(self.advance().value)
                elif self.peek().type == TokenType.STRING:
                    args.append(self.advance().value)
                elif self.peek().type == TokenType.COMMA:
                    self.advance()
                else:
                    break
            self.expect(TokenType.RPAREN)

        return ActionNode(type=action_type, tag=tag, args=args)


def parse_rules(source: str) -> ProgramNode:
    """Parse DSL source code into AST"""
    lexer = Lexer(source)
    tokens = lexer.tokenize()
    parser = Parser(tokens)
    return parser.parse()


# Convenience function for testing
def print_ast(node, indent=0):
    """Pretty print AST"""
    prefix = "  " * indent

    if isinstance(node, ProgramNode):
        print(f"{prefix}Program:")
        for section in node.sections:
            print_ast(section, indent + 1)

    elif isinstance(node, SectionNode):
        print(f"{prefix}@{node.type}:")
        for rule in node.rules:
            print_ast(rule, indent + 1)

    elif isinstance(node, RuleNode):
        print(f"{prefix}rule \"{node.name}\":")
        for stmt in node.statements:
            print_ast(stmt, indent + 1)

    elif isinstance(node, StatementNode):
        if node.pattern:
            print(f"{prefix}{node.type} /{node.pattern}/")
        elif node.condition:
            print(f"{prefix}{node.type} {node.condition}")
        else:
            print(f"{prefix}{node.type}")

        if node.action:
            print_ast(node.action, indent + 1)

        if node.body:
            for stmt in node.body:
                print_ast(stmt, indent + 1)

    elif isinstance(node, ActionNode):
        args_str = ', '.join(node.args) if node.args else ''
        print(f"{prefix}-> {node.type} {node.tag}({args_str})")
