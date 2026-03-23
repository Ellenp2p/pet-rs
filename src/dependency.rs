//! Plugin dependency resolution system.
//!
//! This module provides dependency resolution for WASM plugins.
//! It can parse dependency declarations and resolve the loading order.

use std::collections::{BTreeMap, HashSet};

/// Plugin dependency graph.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Plugin dependencies
    dependencies: BTreeMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new dependency graph.
    pub fn new() -> Self {
        Self {
            dependencies: BTreeMap::new(),
        }
    }

    /// Add a plugin with its dependencies.
    pub fn add_plugin(&mut self, plugin_id: String, deps: Vec<String>) {
        self.dependencies.insert(plugin_id, deps);
    }

    /// Get dependencies for a plugin.
    pub fn get_dependencies(&self, plugin_id: &str) -> Option<&Vec<String>> {
        self.dependencies.get(plugin_id)
    }

    /// Check if a plugin exists in the graph.
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.dependencies.contains_key(plugin_id)
    }

    /// Resolve the loading order using topological sort.
    /// Returns plugins in the order they should be loaded.
    pub fn resolve_loading_order(&self) -> Result<Vec<String>, DependencyError> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut order = Vec::new();

        // Visit each plugin
        for plugin_id in self.dependencies.keys() {
            if !visited.contains(plugin_id) {
                self.visit(plugin_id, &mut visited, &mut temp_visited, &mut order)?;
            }
        }

        // The order is already correct (dependencies first)
        // No need to reverse
        Ok(order)
    }

    /// Visit a plugin and its dependencies (DFS for topological sort).
    fn visit(
        &self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), DependencyError> {
        // Check for circular dependency
        if temp_visited.contains(plugin_id) {
            return Err(DependencyError::CircularDependency(plugin_id.to_string()));
        }

        // Skip if already visited
        if visited.contains(plugin_id) {
            return Ok(());
        }

        // Mark as temporarily visited
        temp_visited.insert(plugin_id.to_string());

        // Visit dependencies first
        if let Some(deps) = self.dependencies.get(plugin_id) {
            for dep in deps {
                if !self.dependencies.contains_key(dep) {
                    return Err(DependencyError::MissingDependency(dep.clone()));
                }
                self.visit(dep, visited, temp_visited, order)?;
            }
        }

        // Mark as visited and add to order
        temp_visited.remove(plugin_id);
        visited.insert(plugin_id.to_string());
        order.push(plugin_id.to_string());

        Ok(())
    }

    /// Check if there are any circular dependencies.
    pub fn has_circular_dependencies(&self) -> bool {
        self.resolve_loading_order().is_err()
    }

    /// Get all plugins that depend on a given plugin.
    pub fn get_dependents(&self, plugin_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for (id, deps) in &self.dependencies {
            if deps.contains(&plugin_id.to_string()) {
                dependents.push(id.clone());
            }
        }
        dependents
    }
}

/// Dependency resolution errors.
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// Circular dependency detected
    CircularDependency(String),
    /// Missing dependency
    MissingDependency(String),
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::CircularDependency(plugin_id) => {
                write!(f, "Circular dependency detected for plugin: {}", plugin_id)
            }
            DependencyError::MissingDependency(plugin_id) => {
                write!(f, "Missing dependency: {}", plugin_id)
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// Plugin dependency manager.
#[derive(Debug, Clone, Default)]
pub struct DependencyManager {
    /// Dependency graph
    graph: DependencyGraph,
}

impl DependencyManager {
    /// Create a new dependency manager.
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Add a plugin with its dependencies.
    pub fn add_plugin(&mut self, plugin_id: String, dependencies: Vec<String>) {
        self.graph.add_plugin(plugin_id, dependencies);
    }

    /// Add a plugin from dependency configuration.
    pub fn add_plugin_from_config(
        &mut self,
        plugin_id: &str,
        dependencies: &[super::config::PluginDependency],
    ) {
        let deps: Vec<String> = dependencies.iter().map(|d| d.plugin_id.clone()).collect();
        self.graph.add_plugin(plugin_id.to_string(), deps);
    }

    /// Get the loading order for all plugins.
    pub fn get_loading_order(&self) -> Result<Vec<String>, DependencyError> {
        self.graph.resolve_loading_order()
    }

    /// Check if a plugin can be loaded (all dependencies are available).
    pub fn can_load_plugin(
        &self,
        plugin_id: &str,
        available_plugins: &HashSet<String>,
    ) -> Result<bool, DependencyError> {
        if let Some(deps) = self.graph.get_dependencies(plugin_id) {
            for dep in deps {
                if !available_plugins.contains(dep) {
                    return Err(DependencyError::MissingDependency(dep.clone()));
                }
            }
        }
        Ok(true)
    }

    /// Get plugins that should be loaded before a given plugin.
    pub fn get_plugins_to_load_before(&self, plugin_id: &str) -> Vec<String> {
        let mut result = Vec::new();
        if let Some(deps) = self.graph.get_dependencies(plugin_id) {
            result.extend(deps.clone());
        }
        result
    }

    /// Get plugins that should be loaded after a given plugin.
    pub fn get_plugins_to_load_after(&self, plugin_id: &str) -> Vec<String> {
        self.graph.get_dependents(plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_no_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_plugin("plugin_a".to_string(), vec![]);
        graph.add_plugin("plugin_b".to_string(), vec![]);

        let order = graph.resolve_loading_order().unwrap();
        assert_eq!(order.len(), 2);
        assert!(order.contains(&"plugin_a".to_string()));
        assert!(order.contains(&"plugin_b".to_string()));
    }

    #[test]
    fn test_dependency_graph_with_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_plugin("plugin_a".to_string(), vec![]);
        graph.add_plugin("plugin_b".to_string(), vec!["plugin_a".to_string()]);
        graph.add_plugin("plugin_c".to_string(), vec!["plugin_b".to_string()]);

        let order = graph.resolve_loading_order().unwrap();
        assert_eq!(order.len(), 3);

        // Check that all plugins are present
        assert!(order.contains(&"plugin_a".to_string()));
        assert!(order.contains(&"plugin_b".to_string()));
        assert!(order.contains(&"plugin_c".to_string()));

        // Check dependencies are satisfied
        let pos_a = order.iter().position(|x| x == "plugin_a").unwrap();
        let pos_b = order.iter().position(|x| x == "plugin_b").unwrap();
        let pos_c = order.iter().position(|x| x == "plugin_c").unwrap();

        // plugin_a should come before plugin_b
        assert!(
            pos_a < pos_b,
            "plugin_a should be loaded before plugin_b, got order: {:?}",
            order
        );
        // plugin_b should come before plugin_c
        assert!(
            pos_b < pos_c,
            "plugin_b should be loaded before plugin_c, got order: {:?}",
            order
        );
    }

    #[test]
    fn test_dependency_graph_circular_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_plugin("plugin_a".to_string(), vec!["plugin_b".to_string()]);
        graph.add_plugin("plugin_b".to_string(), vec!["plugin_a".to_string()]);

        let result = graph.resolve_loading_order();
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_graph_missing_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_plugin("plugin_a".to_string(), vec!["plugin_missing".to_string()]);

        let result = graph.resolve_loading_order();
        assert!(result.is_err());
    }
}
