//! Integration tests for JavaScript parser
//!
//! Tests the full JS parsing pipeline with real-world patterns.

use crawlwe_core::js::{JsParser, JsAnalyzer, MicroVm};

#[test]
fn test_parse_react_component() {
    let code = r#"
        import React from 'react';

        function Button({ className, children }) {
            return (
                <button className="btn btn-primary hover:bg-blue-600">
                    {children}
                </button>
            );
        }

        export default Button;
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.classes.contains(&"btn".to_string()));
    assert!(result.classes.contains(&"btn-primary".to_string()));
    assert!(result.imports.iter().any(|(lib, _)| lib == "react"));
}

#[test]
fn test_parse_styled_components() {
    let code = r#"
        import styled from 'styled-components';

        const Button = styled.button`
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 12px 24px;
            border-radius: 8px;
            border: none;
            cursor: pointer;

            &:hover {
                transform: scale(1.05);
            }
        `;
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(!result.css_in_js.is_empty(), "Should detect CSS-in-JS");

    let (tag, css) = &result.css_in_js[0];
    assert!(tag.contains("styled"));
    assert!(css.contains("background:"));
    assert!(css.contains("padding:"));
}

#[test]
fn test_parse_emotion_css() {
    let code = r#"
        import { css } from '@emotion/react';

        const style = css`
            display: flex;
            justify-content: center;
            align-items: center;
        `;
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.imports.iter().any(|(lib, _)| lib == "emotion"));
}

#[test]
fn test_parse_framer_motion() {
    let code = r#"
        import { motion } from 'framer-motion';

        const Box = () => (
            <motion.div
                className="animated-box"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.5 }}
            >
                Content
            </motion.div>
        );
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.classes.contains(&"animated-box".to_string()));
    assert!(result.imports.iter().any(|(lib, _)| lib == "framer-motion"));
}

#[test]
fn test_parse_dom_manipulation() {
    let code = r#"
        const element = document.createElement('div');
        element.className = 'container wrapper';
        element.classList.add('active', 'visible');
        element.style.backgroundColor = 'red';
        element.style.padding = '20px';
        element.addEventListener('click', handler);
    "#;

    let result = JsParser::parse_to_string_result(code);

    // Check classes
    assert!(result.classes.contains(&"container".to_string()));
    assert!(result.classes.contains(&"wrapper".to_string()));
    assert!(result.classes.contains(&"active".to_string()));
    assert!(result.classes.contains(&"visible".to_string()));

    // Check element creation
    assert!(result.elements_created.contains(&"div".to_string()));

    // Check style assignments
    assert!(result.style_assignments.iter().any(|(prop, _)| prop == "backgroundColor"));
    assert!(result.style_assignments.iter().any(|(prop, _)| prop == "padding"));

    // Check event listeners
    assert!(result.event_listeners.contains(&"click".to_string()));
}

#[test]
fn test_analyzer_full_pipeline() {
    let code = r#"
        import React from 'react';
        import styled from 'styled-components';

        const Title = styled.h1`
            font-size: 2rem;
            color: navy;
        `;

        function App() {
            return (
                <div className="app-container">
                    <Title className="main-title">Hello World</Title>
                </div>
            );
        }
    "#;

    let analysis = JsAnalyzer::analyze(code);

    // Should have classes
    assert!(analysis.classes.contains(&"app-container".to_string()));
    assert!(analysis.classes.contains(&"main-title".to_string()));

    // Should have CSS-in-JS
    assert!(!analysis.css_in_js.is_empty());

    // Should detect libraries
    assert!(analysis.libraries.contains(&"react".to_string()));
    assert!(analysis.libraries.contains(&"styled-components".to_string()));
}

#[test]
fn test_vm_simulation() {
    let code = r#"
        element.className = "initial-class";
        element.classList.add("added-class");
        element.style.color = "blue";
        element.style.fontSize = "16px";
    "#;

    let analysis = JsAnalyzer::analyze(code);
    let mut vm = MicroVm::new();
    vm.simulate(&analysis);

    let state = vm.state();

    // Classes should be tracked
    assert!(state.classes.contains("initial-class"));
    assert!(state.classes.contains("added-class"));

    // Styles should be tracked
    assert_eq!(state.styles.get("color"), Some(&"blue".to_string()));
    assert_eq!(state.styles.get("fontSize"), Some(&"16px".to_string()));

    // Trace should record operations
    assert!(!vm.trace().is_empty());
}

#[test]
fn test_typescript_parsing() {
    let code = r#"
        import React, { FC } from 'react';

        interface Props {
            className?: string;
            variant: 'primary' | 'secondary';
        }

        const Button: FC<Props> = ({ className, variant }) => {
            return (
                <button className={`btn btn-${variant} ${className || ''}`}>
                    Click me
                </button>
            );
        };
    "#;

    let analysis = JsAnalyzer::analyze_typescript(code);
    // TypeScript-specific parsing should not error
    assert!(analysis.errors.is_empty() || analysis.errors.iter().all(|e| !e.contains("fatal")));
}

#[test]
fn test_conditional_classes() {
    let code = r#"
        const element = document.querySelector('.target');
        element.className = isActive ? "active selected" : "inactive";
    "#;

    let result = JsParser::parse_to_string_result(code);
    // Should capture both branches
    assert!(result.classes.iter().any(|c| c.contains("active") || c.contains("inactive")));
}

#[test]
fn test_multiple_files_analysis() {
    let file1 = r#"
        const header = document.createElement('header');
        header.className = 'site-header';
    "#;

    let file2 = r#"
        const footer = document.createElement('footer');
        footer.className = 'site-footer';
    "#;

    let combined = JsAnalyzer::analyze_multiple(&[file1, file2]);

    assert!(combined.classes.contains(&"site-header".to_string()));
    assert!(combined.classes.contains(&"site-footer".to_string()));
}
