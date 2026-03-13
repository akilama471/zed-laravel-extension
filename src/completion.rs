use crate::laravel_scanner::LaravelIndex;
use zed::completion::{CompletionProvider, Suggestion, SuggestionKind};

pub struct LaravelCompletion {
    index: LaravelIndex,
}

impl LaravelCompletion {
    pub fn new(index: LaravelIndex) -> Self {
        Self { index }
    }

    pub fn provide(&self, prefix: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // -------------------
        // Blade Component Completion
        // -------------------
        for comp in &self.index.blade_components {
            if comp.starts_with(prefix) {
                suggestions.push(Suggestion {
                    label: comp.clone(),
                    kind: SuggestionKind::Snippet,
                    insert_text: comp.clone(),
                });
            }
        }

        // -------------------
        // Route Completion
        // -------------------
        for route in &self.index.routes {
            let route_text = format!("route('{}')", route);
            if route_text.starts_with(prefix) {
                suggestions.push(Suggestion {
                    label: route_text.clone(),
                    kind: SuggestionKind::Function,
                    insert_text: route_text,
                });
            }
        }

        // -------------------
        // Model Property Completion
        // -------------------
        for prop in &self.index.model_properties {
            if prop.starts_with(prefix) {
                suggestions.push(Suggestion {
                    label: prop.clone(),
                    kind: SuggestionKind::Field,
                    insert_text: prop.clone(),
                });
            }
        }

        suggestions
    }
}
