//! Plugin loading and registry.

use std::collections::HashMap;

use crate::traits::DomainPlugin;

/// Information about a loaded plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: semver::Version,
    /// Plugin description.
    pub description: String,
    /// Number of strategy preferences.
    pub strategy_count: usize,
    /// Number of action definitions.
    pub action_count: usize,
}

/// Registry that holds loaded plugins and provides access to them.
pub struct PluginRegistry {
    /// Map of plugin name to plugin instance.
    plugins: HashMap<String, Box<dyn DomainPlugin>>,
}

impl PluginRegistry {
    /// Create a new empty PluginRegistry.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin. Returns an error if a plugin with the same name already exists.
    pub fn register(&mut self, plugin: Box<dyn DomainPlugin>) -> Result<(), String> {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' is already registered", name));
        }
        tracing::info!(plugin = %name, version = %plugin.version(), "Registered plugin");
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Get a reference to a plugin by name.
    pub fn get(&self, name: &str) -> Option<&dyn DomainPlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// List all registered plugins with their info.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|p| PluginInfo {
                name: p.name().to_string(),
                version: p.version(),
                description: p.description().to_string(),
                strategy_count: p.strategy_preferences().len(),
                action_count: p.action_extensions().len(),
            })
            .collect()
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Remove a plugin by name.
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn DomainPlugin>> {
        self.plugins.remove(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder for future dynamic plugin loading from shared libraries (.so/.dylib).
pub mod dynamic {
    /// Load a plugin from a shared library path.
    ///
    /// # Safety
    /// This is a placeholder. Dynamic loading will use `libloading` and
    /// require unsafe FFI in a future implementation.
    pub fn load_dynamic(_path: &std::path::Path) -> Result<(), String> {
        Err("Dynamic plugin loading is not yet implemented".to_string())
    }
}

/// Placeholder for future WASM plugin loading.
pub mod wasm {
    /// Load a plugin from a WASM module.
    pub fn load_wasm(_path: &std::path::Path) -> Result<(), String> {
        Err("WASM plugin loading is not yet implemented".to_string())
    }
}
