//! JavaScript content analyzer
//!
//! Analyzes JavaScript for animation libraries, 3D graphics,
//! WebGL/WebGPU usage, UI frameworks, and event handling.

use super::patterns::*;
use super::{
    JsAnalysis, LibraryDetection, ObserverAnalysis, ShaderAnalysis, ThreeJsAnalysis,
    WebGLAnalysis, WebGPUAnalysis,
};

pub struct JsAnalyzer;

impl JsAnalyzer {
    /// Perform complete JavaScript analysis
    pub fn analyze(js: &str) -> JsAnalysis {
        JsAnalysis {
            animation_libraries: Self::detect_animation_libraries(js),
            graphics_libraries: Self::detect_graphics_libraries(js),
            scroll_libraries: Self::detect_scroll_libraries(js),
            ui_frameworks: Self::detect_ui_frameworks(js),
            webgl: Self::analyze_webgl(js),
            webgpu: Self::analyze_webgpu(js),
            observers: Self::analyze_observers(js),
            event_handlers: Self::detect_event_handlers(js),
        }
    }

    /// Detect animation libraries
    fn detect_animation_libraries(js: &str) -> Vec<LibraryDetection> {
        let mut libs = Vec::new();

        let gsap_count = count_matches(&GSAP_PATTERN, js);
        if gsap_count > 0 {
            libs.push(LibraryDetection {
                name: "GSAP".to_string(),
                occurrences: gsap_count,
                confidence: (gsap_count.min(10) as f32) / 10.0,
            });
        }

        let animejs_count = count_matches(&ANIMEJS_PATTERN, js);
        if animejs_count > 0 {
            libs.push(LibraryDetection {
                name: "Anime.js".to_string(),
                occurrences: animejs_count,
                confidence: (animejs_count.min(10) as f32) / 10.0,
            });
        }

        let framer_count = count_matches(&FRAMER_PATTERN, js);
        if framer_count > 0 {
            libs.push(LibraryDetection {
                name: "Framer Motion".to_string(),
                occurrences: framer_count,
                confidence: (framer_count.min(10) as f32) / 10.0,
            });
        }

        let lottie_count = count_matches(&LOTTIE_PATTERN, js);
        if lottie_count > 0 {
            libs.push(LibraryDetection {
                name: "Lottie".to_string(),
                occurrences: lottie_count,
                confidence: (lottie_count.min(10) as f32) / 10.0,
            });
        }

        let velocity_count = count_matches(&VELOCITY_PATTERN, js);
        if velocity_count > 0 {
            libs.push(LibraryDetection {
                name: "Velocity.js".to_string(),
                occurrences: velocity_count,
                confidence: (velocity_count.min(10) as f32) / 10.0,
            });
        }

        libs
    }

    /// Detect 3D/graphics libraries
    fn detect_graphics_libraries(js: &str) -> Vec<LibraryDetection> {
        let mut libs = Vec::new();

        let threejs_count = count_matches(&THREEJS_PATTERN, js);
        if threejs_count > 0 {
            libs.push(LibraryDetection {
                name: "Three.js".to_string(),
                occurrences: threejs_count,
                confidence: (threejs_count.min(20) as f32) / 20.0,
            });
        }

        let babylon_count = count_matches(&BABYLON_PATTERN, js);
        if babylon_count > 0 {
            libs.push(LibraryDetection {
                name: "Babylon.js".to_string(),
                occurrences: babylon_count,
                confidence: (babylon_count.min(20) as f32) / 20.0,
            });
        }

        let pixi_count = count_matches(&PIXI_PATTERN, js);
        if pixi_count > 0 {
            libs.push(LibraryDetection {
                name: "PixiJS".to_string(),
                occurrences: pixi_count,
                confidence: (pixi_count.min(20) as f32) / 20.0,
            });
        }

        let aframe_count = count_matches(&AFRAME_PATTERN, js);
        if aframe_count > 0 {
            libs.push(LibraryDetection {
                name: "A-Frame".to_string(),
                occurrences: aframe_count,
                confidence: (aframe_count.min(10) as f32) / 10.0,
            });
        }

        libs
    }

    /// Detect scroll animation libraries
    fn detect_scroll_libraries(js: &str) -> Vec<LibraryDetection> {
        let mut libs = Vec::new();

        let locomotive_count = count_matches(&LOCOMOTIVE_PATTERN, js);
        if locomotive_count > 0 {
            libs.push(LibraryDetection {
                name: "Locomotive Scroll".to_string(),
                occurrences: locomotive_count,
                confidence: (locomotive_count.min(5) as f32) / 5.0,
            });
        }

        let scrollmagic_count = count_matches(&SCROLLMAGIC_PATTERN, js);
        if scrollmagic_count > 0 {
            libs.push(LibraryDetection {
                name: "ScrollMagic".to_string(),
                occurrences: scrollmagic_count,
                confidence: (scrollmagic_count.min(5) as f32) / 5.0,
            });
        }

        let aos_count = count_matches(&AOS_PATTERN, js);
        if aos_count > 0 {
            libs.push(LibraryDetection {
                name: "AOS".to_string(),
                occurrences: aos_count,
                confidence: (aos_count.min(5) as f32) / 5.0,
            });
        }

        let lenis_count = count_matches(&LENIS_PATTERN, js);
        if lenis_count > 0 {
            libs.push(LibraryDetection {
                name: "Lenis".to_string(),
                occurrences: lenis_count,
                confidence: (lenis_count.min(5) as f32) / 5.0,
            });
        }

        libs
    }

    /// Detect UI frameworks
    fn detect_ui_frameworks(js: &str) -> Vec<LibraryDetection> {
        let mut frameworks = Vec::new();

        let react_count = count_matches(&REACT_PATTERN, js);
        if react_count > 0 {
            frameworks.push(LibraryDetection {
                name: "React".to_string(),
                occurrences: react_count,
                confidence: (react_count.min(20) as f32) / 20.0,
            });
        }

        let vue_count = count_matches(&VUE_PATTERN, js);
        if vue_count > 0 {
            frameworks.push(LibraryDetection {
                name: "Vue".to_string(),
                occurrences: vue_count,
                confidence: (vue_count.min(20) as f32) / 20.0,
            });
        }

        let svelte_count = count_matches(&SVELTE_PATTERN, js);
        if svelte_count > 0 {
            frameworks.push(LibraryDetection {
                name: "Svelte".to_string(),
                occurrences: svelte_count,
                confidence: (svelte_count.min(10) as f32) / 10.0,
            });
        }

        let angular_count = count_matches(&ANGULAR_PATTERN, js);
        if angular_count > 0 {
            frameworks.push(LibraryDetection {
                name: "Angular".to_string(),
                occurrences: angular_count,
                confidence: (angular_count.min(10) as f32) / 10.0,
            });
        }

        let nextjs_count = count_matches(&NEXTJS_PATTERN, js);
        if nextjs_count > 0 {
            frameworks.push(LibraryDetection {
                name: "Next.js".to_string(),
                occurrences: nextjs_count,
                confidence: (nextjs_count.min(5) as f32) / 5.0,
            });
        }

        let nuxt_count = count_matches(&NUXT_PATTERN, js);
        if nuxt_count > 0 {
            frameworks.push(LibraryDetection {
                name: "Nuxt".to_string(),
                occurrences: nuxt_count,
                confidence: (nuxt_count.min(5) as f32) / 5.0,
            });
        }

        frameworks
    }

    /// Analyze WebGL usage
    fn analyze_webgl(js: &str) -> WebGLAnalysis {
        let webgl1 = count_matches(&WEBGL_CONTEXT_PATTERN, js) > 0;
        let webgl2 = count_matches(&WEBGL2_CONTEXT_PATTERN, js) > 0;
        let webgl_rendering = count_matches(&WEBGL_RENDERING_PATTERN, js) > 0;
        let shader_ops = count_matches(&SHADER_CREATE_PATTERN, js) > 0;
        let gl_calls = count_matches(&GL_CALLS_PATTERN, js) > 0;

        let detected = webgl1 || webgl2 || webgl_rendering || shader_ops || gl_calls;

        let version = if webgl2 {
            Some("WebGL2".to_string())
        } else if webgl1 {
            Some("WebGL1".to_string())
        } else {
            None
        };

        let mut patterns_found = Vec::new();
        if webgl1 { patterns_found.push("getContext('webgl')".to_string()); }
        if webgl2 { patterns_found.push("getContext('webgl2')".to_string()); }
        if shader_ops { patterns_found.push("shader operations".to_string()); }
        if gl_calls { patterns_found.push("GL draw calls".to_string()); }

        // Analyze Three.js usage
        let three_js = Self::analyze_threejs(js);

        // Analyze shaders
        let shaders = Self::analyze_shaders(js);

        WebGLAnalysis {
            detected,
            version,
            patterns_found,
            shaders,
            three_js,
        }
    }

    /// Analyze Three.js specific usage
    fn analyze_threejs(js: &str) -> ThreeJsAnalysis {
        let detected = count_matches(&THREEJS_PATTERN, js) > 0;

        if !detected {
            return ThreeJsAnalysis::default();
        }

        let geometries = extract_matches(&THREEJS_GEOMETRY_PATTERN, js);
        let materials = extract_matches(&THREEJS_MATERIAL_PATTERN, js);
        let lights = extract_matches(&THREEJS_LIGHT_PATTERN, js);
        let controls = extract_matches(&THREEJS_CONTROLS_PATTERN, js);

        ThreeJsAnalysis {
            detected,
            geometries,
            materials,
            lights,
            controls,
        }
    }

    /// Analyze shader code
    fn analyze_shaders(js: &str) -> ShaderAnalysis {
        let uniforms: Vec<String> = UNIFORM_PATTERN
            .captures_iter(js)
            .filter_map(|cap| {
                let type_name = cap.get(1)?.as_str();
                let var_name = cap.get(2)?.as_str();
                Some(format!("{} {}", type_name, var_name))
            })
            .take(20)
            .collect();

        let attributes: Vec<String> = ATTRIBUTE_PATTERN
            .captures_iter(js)
            .filter_map(|cap| {
                let type_name = cap.get(1)?.as_str();
                let var_name = cap.get(2)?.as_str();
                Some(format!("{} {}", type_name, var_name))
            })
            .take(20)
            .collect();

        let vertex_shaders = js.matches("gl_Position").count();
        let fragment_shaders = js.matches("gl_FragColor").count();

        ShaderAnalysis {
            vertex_shaders,
            fragment_shaders,
            uniforms,
            attributes,
        }
    }

    /// Analyze WebGPU usage
    fn analyze_webgpu(js: &str) -> WebGPUAnalysis {
        let navigator_gpu = count_matches(&WEBGPU_NAVIGATOR_PATTERN, js) > 0;
        let gpu_device = count_matches(&WEBGPU_DEVICE_PATTERN, js) > 0;
        let gpu_shader = count_matches(&WEBGPU_SHADER_PATTERN, js) > 0;

        let detected = navigator_gpu || gpu_device || gpu_shader;

        let mut patterns_found = Vec::new();
        if navigator_gpu { patterns_found.push("navigator.gpu".to_string()); }
        if gpu_device { patterns_found.push("GPUDevice/Buffer".to_string()); }
        if gpu_shader { patterns_found.push("createShaderModule".to_string()); }

        WebGPUAnalysis {
            detected,
            patterns_found,
        }
    }

    /// Analyze Observer API usage
    fn analyze_observers(js: &str) -> ObserverAnalysis {
        ObserverAnalysis {
            intersection: count_matches(&INTERSECTION_OBSERVER_PATTERN, js),
            mutation: count_matches(&MUTATION_OBSERVER_PATTERN, js),
            resize: count_matches(&RESIZE_OBSERVER_PATTERN, js),
            performance: js.matches("PerformanceObserver").count(),
        }
    }

    /// Detect event handlers
    fn detect_event_handlers(js: &str) -> Vec<String> {
        let mut handlers = Vec::new();

        if count_matches(&SCROLL_EVENT_PATTERN, js) > 0 {
            handlers.push("scroll".to_string());
        }
        if count_matches(&MOUSE_EVENT_PATTERN, js) > 0 {
            handlers.push("mouse".to_string());
        }
        if count_matches(&TOUCH_EVENT_PATTERN, js) > 0 {
            handlers.push("touch".to_string());
        }
        if count_matches(&WHEEL_EVENT_PATTERN, js) > 0 {
            handlers.push("wheel".to_string());
        }
        if count_matches(&RAF_PATTERN, js) > 0 {
            handlers.push("requestAnimationFrame".to_string());
        }
        if count_matches(&WAAPI_PATTERN, js) > 0 {
            handlers.push("Web Animations API".to_string());
        }

        handlers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_threejs() {
        let js = r#"
            const scene = new THREE.Scene();
            const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
            const geometry = new THREE.BoxGeometry();
            const material = new THREE.MeshBasicMaterial();
        "#;
        let analysis = JsAnalyzer::analyze(js);
        assert!(analysis.webgl.three_js.detected);
        assert!(analysis.webgl.three_js.geometries.contains(&"BoxGeometry".to_string()));
    }

    #[test]
    fn test_detect_webgl() {
        let js = r#"
            const canvas = document.querySelector('canvas');
            const gl = canvas.getContext('webgl2');
        "#;
        let analysis = JsAnalyzer::analyze(js);
        assert!(analysis.webgl.detected);
        assert_eq!(analysis.webgl.version, Some("WebGL2".to_string()));
    }
}
