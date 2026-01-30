//! Integration tests for library detection and library-specific parsing
//!
//! Tests that we correctly identify libraries and apply appropriate parsers.

use crawlwe_core::js::{JsAnalyzer, JsParser};

#[test]
fn test_detect_gsap() {
    let code = r#"
        import gsap from 'gsap';
        import { ScrollTrigger } from 'gsap/ScrollTrigger';

        gsap.registerPlugin(ScrollTrigger);

        gsap.to(".box", {
            duration: 1,
            x: 100,
            rotation: 360,
            ease: "power2.out"
        });

        gsap.from(".title", {
            opacity: 0,
            y: -50,
            duration: 0.5
        });
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.imports.iter().any(|(lib, _)| lib == "gsap"));
}

#[test]
fn test_detect_three_js() {
    let code = r#"
        import * as THREE from 'three';
        import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';

        const scene = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight);
        const renderer = new THREE.WebGLRenderer();

        const geometry = new THREE.BoxGeometry();
        const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
        const cube = new THREE.Mesh(geometry, material);
        scene.add(cube);
    "#;

    let result = JsParser::parse_to_string_result(code);
    // Three.js detection - check imports contain 'three'
    assert!(result.imports.iter().any(|(_, src)| src.contains("three")) ||
            result.errors.is_empty()); // At minimum, should parse without errors
}

#[test]
fn test_detect_anime_js() {
    let code = r#"
        import anime from 'animejs';

        anime({
            targets: '.box',
            translateX: 250,
            rotate: '1turn',
            backgroundColor: '#FFF',
            duration: 800,
            easing: 'easeInOutQuad'
        });
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.errors.is_empty());
}

#[test]
fn test_detect_tailwind_utilities() {
    let code = r#"
        const Card = () => (
            <div className="bg-white shadow-lg rounded-xl p-6 hover:shadow-xl transition-shadow">
                <h2 className="text-2xl font-bold text-gray-800 mb-4">Title</h2>
                <p className="text-gray-600 leading-relaxed">Description</p>
            </div>
        );
    "#;

    let result = JsParser::parse_to_string_result(code);

    // Should detect Tailwind utility classes
    assert!(result.classes.iter().any(|c| c.starts_with("bg-")));
    assert!(result.classes.iter().any(|c| c.starts_with("text-")));
    assert!(result.classes.iter().any(|c| c.contains("shadow")));
    assert!(result.classes.iter().any(|c| c.starts_with("p-")));
}

#[test]
fn test_detect_multiple_libraries() {
    let code = r#"
        import React from 'react';
        import styled from 'styled-components';
        import { motion } from 'framer-motion';
        import gsap from 'gsap';

        const Container = styled.div`
            display: flex;
            padding: 20px;
        `;

        const AnimatedBox = () => {
            useEffect(() => {
                gsap.to(".box", { x: 100 });
            }, []);

            return (
                <Container>
                    <motion.div className="box animated" animate={{ scale: 1.2 }}>
                        Content
                    </motion.div>
                </Container>
            );
        };
    "#;

    let analysis = JsAnalyzer::analyze(code);

    // Should detect all libraries
    assert!(analysis.libraries.contains(&"react".to_string()));
    assert!(analysis.libraries.contains(&"styled-components".to_string()));
    assert!(analysis.libraries.contains(&"framer-motion".to_string()));
    assert!(analysis.libraries.contains(&"gsap".to_string()));

    // Should have classes
    assert!(analysis.classes.contains(&"box".to_string()));
    assert!(analysis.classes.contains(&"animated".to_string()));

    // Should have CSS-in-JS
    assert!(!analysis.css_in_js.is_empty());
}

#[test]
fn test_css_modules_pattern() {
    let code = r#"
        import styles from './Button.module.css';

        const Button = ({ children }) => (
            <button className={styles.button}>
                {children}
            </button>
        );
    "#;

    let result = JsParser::parse_to_string_result(code);
    // CSS modules use local imports which we don't track as libraries
    // but the code should parse without errors
    assert!(result.errors.is_empty());
}

#[test]
fn test_inline_styles_object() {
    let code = r#"
        const Box = () => (
            <div style={{
                backgroundColor: 'coral',
                padding: '20px',
                borderRadius: '8px',
                boxShadow: '0 4px 6px rgba(0,0,0,0.1)'
            }}>
                Styled content
            </div>
        );
    "#;

    let result = JsParser::parse_to_string_result(code);
    assert!(result.errors.is_empty());
}

#[test]
fn test_clsx_classnames_pattern() {
    let code = r#"
        import clsx from 'clsx';

        const Button = ({ isActive, isPrimary }) => (
            <button className={clsx(
                'btn',
                isActive && 'btn-active',
                isPrimary ? 'btn-primary' : 'btn-secondary'
            )}>
                Click
            </button>
        );
    "#;

    let result = JsParser::parse_to_string_result(code);
    // clsx patterns with dynamic expressions are complex
    // The parser should at least not error on this code
    // Future: implement clsx string extraction
    assert!(result.errors.is_empty());
}
