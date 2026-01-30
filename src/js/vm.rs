//! Micro VM for JavaScript execution simulation
//!
//! Simulates the effects of JavaScript execution to understand rendering.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::analyzer::JsAnalysisResult;

/// State tracked by the micro VM
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VmState {
    /// Active CSS classes
    pub classes: HashSet<String>,

    /// Active inline styles
    pub styles: HashMap<String, String>,

    /// Injected CSS
    pub css: Vec<String>,

    /// Created elements
    pub elements: Vec<String>,

    /// Execution trace
    pub trace: Vec<String>,
}

/// Micro VM for JS execution simulation
#[derive(Debug, Default)]
pub struct MicroVm {
    state: VmState,
}

impl MicroVm {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current state
    pub fn state(&self) -> &VmState {
        &self.state
    }

    /// Simulate execution from analysis results
    pub fn simulate(&mut self, analysis: &JsAnalysisResult) {
        // Add all discovered classes
        for class in &analysis.classes {
            self.state.classes.insert(class.clone());
            self.state.trace.push(format!("class: {}", class));
        }

        // Add styles
        for (prop, val) in &analysis.styles {
            self.state.styles.insert(prop.clone(), val.clone());
            self.state.trace.push(format!("style: {} = {}", prop, val));
        }

        // Add CSS-in-JS
        for css in &analysis.css_in_js {
            self.state.css.push(css.clone());
            self.state.trace.push(format!("css-in-js: {} bytes", css.len()));
        }

        // Add created elements
        for elem in &analysis.elements_created {
            self.state.elements.push(elem.clone());
            self.state.trace.push(format!("createElement: {}", elem));
        }
    }

    /// Get all active classes
    pub fn get_classes(&self) -> Vec<String> {
        self.state.classes.iter().cloned().collect()
    }

    /// Generate CSS from simulated state
    pub fn generate_css(&self) -> String {
        let mut css = String::new();

        // CSS-in-JS
        for block in &self.state.css {
            css.push_str(block);
            css.push_str("\n\n");
        }

        // Styles as utility class
        if !self.state.styles.is_empty() {
            css.push_str(".vm-styles {\n");
            for (prop, val) in &self.state.styles {
                css.push_str(&format!("  {}: {};\n", prop, val));
            }
            css.push_str("}\n");
        }

        css
    }

    /// Get execution trace
    pub fn trace(&self) -> &[String] {
        &self.state.trace
    }
}

/// Combined result from analysis and VM simulation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsExecutionResult {
    /// All unique classes
    pub classes: Vec<String>,

    /// All styles
    pub styles: HashMap<String, String>,

    /// All CSS (including CSS-in-JS)
    pub css: String,

    /// Libraries detected
    pub libraries: Vec<String>,

    /// Execution summary
    pub summary: ExecutionSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_classes: usize,
    pub total_styles: usize,
    pub css_in_js_blocks: usize,
    pub elements_created: usize,
    pub event_listeners: usize,
}

impl JsExecutionResult {
    /// Create from analysis and VM
    pub fn from_analysis(analysis: &JsAnalysisResult, vm: &MicroVm) -> Self {
        let mut classes: HashSet<String> = HashSet::new();
        classes.extend(analysis.classes.iter().cloned());
        classes.extend(vm.state.classes.iter().cloned());

        let mut styles = HashMap::new();
        for (k, v) in &analysis.styles {
            styles.insert(k.clone(), v.clone());
        }
        for (k, v) in &vm.state.styles {
            styles.insert(k.clone(), v.clone());
        }

        let css = vm.generate_css();

        Self {
            classes: classes.into_iter().collect(),
            styles,
            css,
            libraries: analysis.libraries.clone(),
            summary: ExecutionSummary {
                total_classes: analysis.classes.len(),
                total_styles: analysis.styles.len(),
                css_in_js_blocks: analysis.css_in_js.len(),
                elements_created: analysis.elements_created.len(),
                event_listeners: analysis.event_listeners.len(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::analyzer::JsAnalyzer;

    #[test]
    fn test_vm_simulation() {
        let code = r#"
            element.className = "active";
            element.style.color = "red";
        "#;

        let analysis = JsAnalyzer::analyze(code);
        let mut vm = MicroVm::new();
        vm.simulate(&analysis);

        assert!(vm.state.classes.contains("active"));
        assert_eq!(vm.state.styles.get("color"), Some(&"red".to_string()));
    }
}
