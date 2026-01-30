//! Integration tests for JS to CSS converter

use crawlwe_core::js::JsToCssConverter;

#[test]
fn test_styled_components_real_example() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        const Container = styled.div`
            display: flex;
            flex-direction: column;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        `;

        const Button = styled.button`
            background-color: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            padding: 10px 20px;
            cursor: pointer;

            &:hover {
                background-color: #0056b3;
            }
        `;

        const Card = styled.article`
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            padding: 24px;
        `;
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nStats:");
    println!("  styled_components_found: {}", result.stats.styled_components_found);

    assert_eq!(result.stats.styled_components_found, 3, "Should find 3 styled components");
    assert!(result.css.contains("display:"), "Should have display property");
    assert!(result.css.contains("background-color:"), "Should have background-color");
}

#[test]
fn test_gsap_animation_conversion() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        // Fade in animation
        gsap.to(".hero-text", {
            opacity: 1,
            y: 0,
            duration: 1.2,
            ease: "power3.out"
        });

        // Slide from left
        gsap.from(".sidebar", {
            x: -100,
            opacity: 0,
            duration: 0.8
        });

        // Complex fromTo
        gsap.fromTo(".card",
            { scale: 0.8, opacity: 0 },
            { scale: 1, opacity: 1, duration: 0.5 }
        );
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nStats:");
    println!("  gsap_animations_found: {}", result.stats.gsap_animations_found);
    println!("  keyframes_generated: {}", result.keyframes.len());

    assert_eq!(result.stats.gsap_animations_found, 3, "Should find 3 GSAP animations");
    assert!(result.css.contains("@keyframes"), "Should generate keyframes");
    assert!(result.css.contains("translateX") || result.css.contains("translateY"), "Should have transform");
}

#[test]
fn test_framer_motion_conversion() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        <motion.div
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
        />

        <motion.button
            animate={{ scale: 1.1, rotate: 5 }}
        />
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nStats:");
    println!("  framer_animations_found: {}", result.stats.framer_animations_found);

    assert!(result.stats.framer_animations_found >= 1, "Should find Framer Motion animations");
}

#[test]
fn test_keyframes_tagged_template() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        const fadeIn = keyframes`
            from {
                opacity: 0;
                transform: translateY(20px);
            }
            to {
                opacity: 1;
                transform: translateY(0);
            }
        `;

        const pulse = keyframes`
            0% { transform: scale(1); }
            50% { transform: scale(1.05); }
            100% { transform: scale(1); }
        `;
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nKeyframes generated: {}", result.stats.keyframes_generated);

    assert_eq!(result.stats.keyframes_generated, 2, "Should extract 2 keyframe animations");
    assert!(result.css.contains("@keyframes"), "Should have @keyframes");
}

#[test]
fn test_inline_style_conversion() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        <div style={{ backgroundColor: "red", padding: "20px", borderRadius: "8px" }} />
        <span style={{ color: "blue", fontSize: "16px" }} />
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nInline styles converted: {}", result.stats.inline_styles_converted);

    assert!(result.stats.inline_styles_converted >= 1, "Should convert inline styles");
    assert!(result.css.contains("background-color"), "Should convert camelCase to kebab-case");
}

#[test]
fn test_css_tagged_template() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        const styles = css`
            .container {
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                gap: 20px;
            }
        `;

        const buttonStyles = css`
            padding: 10px;
            margin: 5px;
            border: 1px solid #ccc;
        `;
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);
    println!("\nEmotion blocks found: {}", result.stats.emotion_blocks_found);

    assert!(result.stats.emotion_blocks_found >= 1, "Should find css`` blocks");
}

#[test]
fn test_global_styles() {
    let mut converter = JsToCssConverter::new();

    let js = r#"
        const GlobalStyle = createGlobalStyle`
            * {
                box-sizing: border-box;
                margin: 0;
                padding: 0;
            }

            body {
                font-family: 'Inter', sans-serif;
                line-height: 1.6;
            }
        `;
    "#;

    let result = converter.convert(js);

    println!("Generated CSS:\n{}", result.css);

    assert!(result.css.contains("box-sizing"), "Should extract global styles");
    assert!(result.css.contains("font-family"), "Should have body styles");
}

#[test]
fn test_complete_conversion_flow() {
    let mut converter = JsToCssConverter::new();

    // Mix of different JS styling patterns
    let js = r#"
        // Styled-components
        const Header = styled.header`
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 40px;
            color: white;
        `;

        // GSAP animation
        gsap.to(".header-title", {
            opacity: 1,
            y: 0,
            duration: 1
        });

        // Keyframes
        const bounce = keyframes`
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-20px); }
        `;

        // Inline styles in JSX
        <div style={{ marginTop: "20px", textAlign: "center" }} />
    "#;

    let result = converter.convert(js);

    println!("=== Complete Conversion Result ===");
    println!("\nGenerated CSS ({} bytes):\n{}", result.css.len(), result.css);
    println!("\n=== Statistics ===");
    println!("  styled_components: {}", result.stats.styled_components_found);
    println!("  gsap_animations: {}", result.stats.gsap_animations_found);
    println!("  keyframes_generated: {}", result.stats.keyframes_generated);
    println!("  inline_styles: {}", result.stats.inline_styles_converted);
    println!("  dynamic_skipped: {}", result.stats.dynamic_values_skipped);

    // Should have content from multiple sources
    assert!(!result.css.is_empty(), "Should generate CSS");
    assert!(result.stats.styled_components_found > 0, "Should find styled-components");
    assert!(result.stats.gsap_animations_found > 0 || result.stats.keyframes_generated > 0, "Should find animations");
}
