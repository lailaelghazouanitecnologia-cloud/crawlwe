//! JavaScript parser using OXC
//!
//! Parses JavaScript/TypeScript code into AST for analysis.
//! OXC is a fast Rust-native parser, faster than SWC.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Parsed JavaScript result
pub struct ParsedJs<'a> {
    pub program: oxc_ast::ast::Program<'a>,
    pub source: &'a str,
}

/// JavaScript parser wrapper around OXC
pub struct JsParser;

impl JsParser {
    /// Parse JavaScript code into AST
    /// Returns owned data since OXC uses arena allocation
    pub fn parse_to_string_result(code: &str) -> JsParseResult {
        let allocator = Allocator::default();
        // Use JSX-enabled source type for React code
        let source_type = SourceType::jsx();

        let parser_result = Parser::new(&allocator, code, source_type).parse();

        let mut result = JsParseResult::new();

        if !parser_result.errors.is_empty() {
            for error in &parser_result.errors {
                result.errors.push(format!("{:?}", error));
            }
        }

        // Extract info from AST before allocator is dropped
        Self::extract_from_program(&parser_result.program, &mut result);

        result
    }

    /// Parse TypeScript code
    pub fn parse_typescript(code: &str) -> JsParseResult {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx(); // TypeScript with JSX

        let parser_result = Parser::new(&allocator, code, source_type).parse();

        let mut result = JsParseResult::new();

        if !parser_result.errors.is_empty() {
            for error in &parser_result.errors {
                result.errors.push(format!("{:?}", error));
            }
        }

        Self::extract_from_program(&parser_result.program, &mut result);

        result
    }

    /// Extract information from the parsed program
    fn extract_from_program(program: &oxc_ast::ast::Program, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        for stmt in &program.body {
            Self::extract_from_statement(stmt, result);
        }
    }

    fn extract_from_statement(stmt: &oxc_ast::ast::Statement, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                Self::extract_from_expression(&expr_stmt.expression, result);
            }
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    if let Some(init) = &decl.init {
                        Self::extract_from_expression(init, result);
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        Self::extract_from_statement(stmt, result);
                    }
                }
            }
            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    Self::extract_from_expression(arg, result);
                }
            }
            Statement::IfStatement(if_stmt) => {
                Self::extract_from_expression(&if_stmt.test, result);
                Self::extract_from_statement(&if_stmt.consequent, result);
                if let Some(alt) = &if_stmt.alternate {
                    Self::extract_from_statement(alt, result);
                }
            }
            Statement::BlockStatement(block) => {
                for stmt in &block.body {
                    Self::extract_from_statement(stmt, result);
                }
            }
            Statement::ForStatement(for_stmt) => {
                Self::extract_from_statement(&for_stmt.body, result);
            }
            Statement::WhileStatement(while_stmt) => {
                Self::extract_from_statement(&while_stmt.body, result);
            }
            Statement::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(body) = &func.body {
                            for stmt in &body.statements {
                                Self::extract_from_statement(stmt, result);
                            }
                        }
                    }
                    ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                        Self::extract_from_function_body(&arrow.body, result);
                    }
                    _ => {}
                }
            }
            Statement::ImportDeclaration(import) => {
                let source = import.source.value.to_string();
                // Detect libraries from imports
                if source.contains("styled-components") {
                    result.imports.push(("styled-components".to_string(), source));
                } else if source.contains("@emotion") {
                    result.imports.push(("emotion".to_string(), source));
                } else if source.contains("framer-motion") {
                    result.imports.push(("framer-motion".to_string(), source));
                } else if source.contains("gsap") {
                    result.imports.push(("gsap".to_string(), source));
                } else if source.contains("react") {
                    result.imports.push(("react".to_string(), source));
                }
            }
            _ => {}
        }
    }

    fn extract_from_function_body(body: &oxc_ast::ast::FunctionBody, result: &mut JsParseResult) {
        for stmt in &body.statements {
            Self::extract_from_statement(stmt, result);
        }
    }

    fn extract_from_expression(expr: &oxc_ast::ast::Expression, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        match expr {
            // Assignment: element.className = "..."
            Expression::AssignmentExpression(assign) => {
                Self::extract_from_assignment(assign, result);
            }

            // Call: classList.add("..."), createElement("div")
            Expression::CallExpression(call) => {
                Self::extract_from_call(call, result);
            }

            // Arrow function
            Expression::ArrowFunctionExpression(arrow) => {
                Self::extract_from_function_body(&arrow.body, result);
            }

            // Function expression
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        Self::extract_from_statement(stmt, result);
                    }
                }
            }

            // Conditional: condition ? "class1" : "class2"
            Expression::ConditionalExpression(cond) => {
                if let Some(true_val) = Self::extract_string_value(&cond.consequent) {
                    result.classes.push(true_val);
                }
                if let Some(false_val) = Self::extract_string_value(&cond.alternate) {
                    result.classes.push(false_val);
                }
            }

            // Template literal: `class-${var}`
            Expression::TemplateLiteral(tpl) => {
                for quasi in &tpl.quasis {
                    let text = quasi.value.raw.to_string();
                    if text.contains("class") || text.contains("-") {
                        result.templates.push(text);
                    }
                }
            }

            // Tagged template: styled.div`...`, css`...`
            Expression::TaggedTemplateExpression(tagged) => {
                Self::extract_from_tagged_template(tagged, result);
            }

            // JSX element
            Expression::JSXElement(jsx) => {
                Self::extract_from_jsx(jsx, result);
            }

            // Object literal
            Expression::ObjectExpression(obj) => {
                Self::extract_from_object(obj, result);
            }

            // Sequence
            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    Self::extract_from_expression(expr, result);
                }
            }

            // Parenthesized
            Expression::ParenthesizedExpression(paren) => {
                Self::extract_from_expression(&paren.expression, result);
            }

            _ => {}
        }
    }

    fn extract_from_assignment(assign: &oxc_ast::ast::AssignmentExpression, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        // Check for className assignment
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            if ident.name.as_str() == "className" {
                if let Some(value) = Self::extract_string_value(&assign.right) {
                    for class in value.split_whitespace() {
                        result.classes.push(class.to_string());
                    }
                }
            }
        }

        // Check for member expression: element.className = "..."
        if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
            let prop_name = member.property.name.as_str();

            if prop_name == "className" {
                if let Some(value) = Self::extract_string_value(&assign.right) {
                    for class in value.split_whitespace() {
                        result.classes.push(class.to_string());
                    }
                    result.class_assignments.push(format!("className = \"{}\"", value));
                }
            }

            // style.X = "Y"
            if let Expression::StaticMemberExpression(parent) = &member.object {
                if parent.property.name.as_str() == "style" {
                    if let Some(value) = Self::extract_string_value(&assign.right) {
                        result.style_assignments.push((prop_name.to_string(), value));
                    }
                }
            }

            // innerHTML / outerHTML
            if prop_name == "innerHTML" || prop_name == "outerHTML" {
                if let Some(value) = Self::extract_string_value(&assign.right) {
                    result.html_assignments.push(value);
                }
            }
        }

        // Analyze right side
        Self::extract_from_expression(&assign.right, result);
    }

    fn extract_from_call(call: &oxc_ast::ast::CallExpression, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        // Check for classList methods
        if let Expression::StaticMemberExpression(member) = &call.callee {
            let method_name = member.property.name.as_str();

            // classList.add/remove/toggle
            if let Expression::StaticMemberExpression(parent) = &member.object {
                if parent.property.name.as_str() == "classList" {
                    let args: Vec<String> = call.arguments.iter()
                        .filter_map(|arg| {
                            if let Argument::StringLiteral(s) = arg {
                                Some(s.value.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    match method_name {
                        "add" => {
                            for class in &args {
                                result.classes.push(class.clone());
                            }
                            result.classlist_calls.push(format!("classList.add({:?})", args));
                        }
                        "remove" => {
                            result.classlist_calls.push(format!("classList.remove({:?})", args));
                        }
                        "toggle" => {
                            for class in &args {
                                result.classes.push(class.clone());
                            }
                            result.classlist_calls.push(format!("classList.toggle({:?})", args));
                        }
                        "replace" if args.len() >= 2 => {
                            result.classes.push(args[1].clone());
                            result.classlist_calls.push(format!("classList.replace({:?})", args));
                        }
                        _ => {}
                    }
                }
            }

            // createElement
            if method_name == "createElement" {
                if let Some(Argument::StringLiteral(tag)) = call.arguments.first() {
                    result.elements_created.push(tag.value.to_string());
                }
            }

            // addEventListener
            if method_name == "addEventListener" {
                if let Some(Argument::StringLiteral(event)) = call.arguments.first() {
                    result.event_listeners.push(event.value.to_string());
                }
            }

            // setAttribute
            if method_name == "setAttribute" && call.arguments.len() >= 2 {
                if let (Some(Argument::StringLiteral(attr)), Some(Argument::StringLiteral(value))) =
                    (call.arguments.get(0), call.arguments.get(1))
                {
                    if attr.value.as_str() == "class" {
                        for class in value.value.to_string().split_whitespace() {
                            result.classes.push(class.to_string());
                        }
                    }
                }
            }
        }

        // Analyze arguments
        for arg in &call.arguments {
            if let Argument::SpreadElement(spread) = arg {
                Self::extract_from_expression(&spread.argument, result);
            } else if let Some(expr) = arg.as_expression() {
                Self::extract_from_expression(expr, result);
            }
        }
    }

    fn extract_from_tagged_template(tagged: &oxc_ast::ast::TaggedTemplateExpression, result: &mut JsParseResult) {
        // Get tag name
        let tag_name = match &tagged.tag {
            oxc_ast::ast::Expression::Identifier(id) => id.name.to_string(),
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                format!("styled.{}", member.property.name)
            }
            _ => "tagged".to_string(),
        };

        // Extract CSS content
        let mut css = String::new();
        for quasi in &tagged.quasi.quasis {
            css.push_str(&quasi.value.raw.to_string());
        }

        if !css.is_empty() && (tag_name.starts_with("styled") || tag_name == "css") {
            result.css_in_js.push((tag_name, css));
        }
    }

    fn extract_from_jsx(jsx: &oxc_ast::ast::JSXElement, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        // Extract attributes
        for attr in &jsx.opening_element.attributes {
            if let JSXAttributeItem::Attribute(jsx_attr) = attr {
                if let JSXAttributeName::Identifier(name) = &jsx_attr.name {
                    let attr_name = name.name.as_str();

                    // className="..."
                    if attr_name == "className" || attr_name == "class" {
                        if let Some(value) = &jsx_attr.value {
                            if let JSXAttributeValue::StringLiteral(s) = value {
                                for class in s.value.to_string().split_whitespace() {
                                    result.classes.push(class.to_string());
                                }
                                result.jsx_classnames.push(s.value.to_string());
                            }
                        }
                    }

                    // style={{...}}
                    if attr_name == "style" {
                        if let Some(JSXAttributeValue::ExpressionContainer(container)) = &jsx_attr.value {
                            if let JSXExpression::ObjectExpression(obj) = &container.expression {
                                Self::extract_from_object(obj, result);
                            }
                        }
                    }
                }
            }
        }

        // Extract from children
        for child in &jsx.children {
            if let JSXChild::Element(child_elem) = child {
                Self::extract_from_jsx(child_elem, result);
            }
        }
    }

    fn extract_from_object(obj: &oxc_ast::ast::ObjectExpression, result: &mut JsParseResult) {
        use oxc_ast::ast::*;

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                if let PropertyKey::StaticIdentifier(key) = &prop.key {
                    let key_name = key.name.as_str();

                    if key_name == "className" {
                        if let Some(value) = Self::extract_string_value(&prop.value) {
                            for class in value.split_whitespace() {
                                result.classes.push(class.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    fn extract_string_value(expr: &oxc_ast::ast::Expression) -> Option<String> {
        use oxc_ast::ast::*;

        match expr {
            Expression::StringLiteral(s) => Some(s.value.to_string()),
            Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => {
                tpl.quasis.first().map(|q| q.value.raw.to_string())
            }
            _ => None,
        }
    }
}

/// Result of parsing JavaScript
#[derive(Debug, Clone, Default)]
pub struct JsParseResult {
    /// CSS classes found
    pub classes: Vec<String>,

    /// className assignments
    pub class_assignments: Vec<String>,

    /// classList method calls
    pub classlist_calls: Vec<String>,

    /// Style property assignments (property, value)
    pub style_assignments: Vec<(String, String)>,

    /// HTML content assignments (innerHTML, etc)
    pub html_assignments: Vec<String>,

    /// Elements created via createElement
    pub elements_created: Vec<String>,

    /// Event listeners added
    pub event_listeners: Vec<String>,

    /// CSS-in-JS blocks (tag, css)
    pub css_in_js: Vec<(String, String)>,

    /// JSX className attributes
    pub jsx_classnames: Vec<String>,

    /// Template literals that look like classes
    pub templates: Vec<String>,

    /// Imports (library name, source)
    pub imports: Vec<(String, String)>,

    /// Parse errors
    pub errors: Vec<String>,
}

impl JsParseResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all unique classes
    pub fn unique_classes(&self) -> Vec<String> {
        let mut unique: Vec<String> = self.classes.clone();
        unique.sort();
        unique.dedup();
        unique
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"
            const x = 1;
            function hello() {
                console.log("Hello");
            }
        "#;

        let result = JsParser::parse_to_string_result(code);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_classname() {
        let code = r#"element.className = "active hover";"#;
        let result = JsParser::parse_to_string_result(code);
        assert!(result.classes.contains(&"active".to_string()));
        assert!(result.classes.contains(&"hover".to_string()));
    }

    #[test]
    fn test_parse_classlist() {
        let code = r#"element.classList.add("visible", "active");"#;
        let result = JsParser::parse_to_string_result(code);
        assert!(result.classes.contains(&"visible".to_string()));
        assert!(result.classes.contains(&"active".to_string()));
    }
}
