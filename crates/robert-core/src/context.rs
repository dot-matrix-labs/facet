use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Context not found: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<String>,
    pub included_paths: Vec<String>,
}

pub struct ContextManager {
    contexts: std::sync::RwLock<HashMap<String, Context>>,
    active_context_id: std::sync::RwLock<Option<String>>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    pub fn new() -> Self {
        let manager = Self {
            contexts: std::sync::RwLock::new(HashMap::new()),
            active_context_id: std::sync::RwLock::new(None),
        };

        // Initialize defaults
        let personal = Context {
            id: "personal".to_string(),
            name: "Personal".to_string(),
            description: "Personal projects and interests".to_string(),
            rules: vec![],
            included_paths: vec![],
        };

        let work = Context {
            id: "work".to_string(),
            name: "Work".to_string(),
            description: "Work related documents".to_string(),
            rules: vec![],
            included_paths: vec![],
        };

        manager.create_context(personal).unwrap();
        manager.create_context(work).unwrap();

        manager
    }

    pub fn create_context(&self, context: Context) -> Result<(), ContextError> {
        let mut contexts = self.contexts.write().unwrap();
        if contexts.contains_key(&context.id) {
            return Err(ContextError::AlreadyExists(context.id));
        }
        contexts.insert(context.id.clone(), context);
        Ok(())
    }

    pub fn get_context(&self, id: &str) -> Result<Context, ContextError> {
        let contexts = self.contexts.read().unwrap();
        contexts
            .get(id)
            .cloned()
            .ok_or(ContextError::NotFound(id.to_string()))
    }

    pub fn set_active_context(&self, id: &str) -> Result<(), ContextError> {
        let contexts = self.contexts.read().unwrap();
        if !contexts.contains_key(id) {
            return Err(ContextError::NotFound(id.to_string()));
        }
        let mut active = self.active_context_id.write().unwrap();
        *active = Some(id.to_string());
        Ok(())
    }

    pub fn get_active_context(&self) -> Option<Context> {
        let active_id = self.active_context_id.read().unwrap();
        if let Some(id) = active_id.as_ref() {
            self.get_context(id).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_management() {
        let manager = ContextManager::new();

        // Check defaults
        let personal = manager.get_context("personal").unwrap();
        assert_eq!(personal.name, "Personal");

        // Create new
        let new_ctx = Context {
            id: "project-alpha".to_string(),
            name: "Project Alpha".to_string(),
            description: "Top secret".to_string(),
            rules: vec![],
            included_paths: vec![],
        };
        manager.create_context(new_ctx.clone()).unwrap();

        // Switch
        manager.set_active_context("project-alpha").unwrap();
        let active = manager.get_active_context().unwrap();
        assert_eq!(active.id, "project-alpha");
    }
}
