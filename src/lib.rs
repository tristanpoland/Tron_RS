use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TronError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Missing placeholder: {0}")]
    MissingPlaceholder(String),
    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type Result<T> = std::result::Result<T, TronError>;

/// A reference to a template that can be executed or composed
#[derive(Debug, Clone)]
pub struct TronRef {
    template: TronTemplate,
    dependencies: Vec<String>,
}

impl TronRef {
    /// Create a new template reference
    pub fn new(template: TronTemplate) -> Self {
        Self {
            template,
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency that will be included in rust-script execution
    pub fn with_dependency(mut self, dependency: &str) -> Self {
        self.dependencies.push(dependency.to_string());
        self
    }

    /// Get a reference to the inner template
    pub fn inner(&self) -> &TronTemplate {
        &self.template
    }

    /// Get a mutable reference to the inner template
    pub fn inner_mut(&mut self) -> &mut TronTemplate {
        &mut self.template
    }

    /// Set a placeholder value
    pub fn set(&mut self, placeholder: &str, value: &str) -> Result<()> {
        self.template.set(placeholder, value)
    }

    /// Set a placeholder to use another template
    pub fn set_ref(&mut self, placeholder: &str, template_ref: TronRef) -> Result<()> {
        // First render the template we're inserting
        let rendered = template_ref.template.render()?;
        
        // Set the rendered content as the placeholder value
        self.template.set(placeholder, &rendered)?;

        // Merge dependencies
        self.dependencies.extend(template_ref.dependencies);
        
        Ok(())
    }

    /// Execute the template with rust-script
    #[cfg(feature = "execute")]
    pub async fn execute(&self) -> Result<String> {
        use std::process::Command;
        use tempfile::NamedTempFile;
        use std::io::Write;
        use which::which;

        which("rust-script").map_err(|_| {
            TronError::ExecutionError("rust-script not found. Install with: cargo install rust-script".into())
        })?;

        let rendered = self.template.render()?;
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| TronError::ExecutionError(format!("Failed to create temp file: {}", e)))?;
        
        let mut script_content = String::new();
        for dep in &self.dependencies {
            script_content.push_str(&format!("//! ```cargo\n//! [dependencies]\n//! {} \n//! ```\n", dep));
        }
        script_content.push_str(&rendered);

        temp_file.write_all(script_content.as_bytes())
            .map_err(|e| TronError::ExecutionError(format!("Failed to write temp file: {}", e)))?;

        let output = Command::new("rust-script")
            .arg(temp_file.path())
            .output()
            .map_err(|e| TronError::ExecutionError(format!("Failed to execute script: {}", e)))?;

        if !output.status.success() {
            return Err(TronError::ExecutionError(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Render the template to a string
    pub fn render(&self) -> Result<String> {
        self.template.render()
    }
}

#[derive(Debug, Clone)]
pub struct TronTemplate {
    content: String,
    placeholders: HashMap<String, String>,
    path: Option<PathBuf>,
}

impl TronTemplate {
    /// Create a new template from a string
    pub fn new(content: &str) -> Result<Self> {
        let placeholders = Self::extract_placeholders(content)?;
        Ok(Self {
            content: content.to_string(),
            placeholders,
            path: None,
        })
    }

    /// Load a template from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let mut template = Self::new(&content)?;
        template.path = Some(path.as_ref().to_path_buf());
        Ok(template)
    }

    fn extract_placeholders(content: &str) -> Result<HashMap<String, String>> {
        let mut placeholders = HashMap::new();
        let pattern = regex::Regex::new(r"@\[([^]]+)\]@").unwrap();
        
        for capture in pattern.captures_iter(content) {
            let placeholder = capture.get(1).unwrap().as_str().trim();
            placeholders.insert(placeholder.to_string(), String::new());
        }
        
        Ok(placeholders)
    }

    /// Set a placeholder value
    pub fn set(&mut self, placeholder: &str, value: &str) -> Result<()> {
        if !self.placeholders.contains_key(placeholder) {
            return Err(TronError::MissingPlaceholder(placeholder.to_string()));
        }
        self.placeholders.insert(placeholder.to_string(), value.to_string());
        Ok(())
    }

    /// Render the template
    pub fn render(&self) -> Result<String> {
        let mut result = self.content.clone();
        
        for (placeholder, value) in &self.placeholders {
            let pattern = format!("@[{}]@", placeholder);
            if value.is_empty() {
                return Err(TronError::MissingPlaceholder(placeholder.clone()));
            }
            result = result.replace(&pattern, value);
        }
        
        Ok(result)
    }
}

/// Assemble multiple templates together
#[derive(Debug)]
pub struct TronAssembler {
    templates: Vec<TronRef>,
}

impl TronAssembler {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Add a template reference to the assembler
    pub fn add_template(&mut self, template: TronRef) {
        self.templates.push(template);
    }

    /// Set a value for a placeholder across all templates
    pub fn set_global(&mut self, placeholder: &str, value: &str) -> Result<()> {
        for template in &mut self.templates {
            if template.inner().placeholders.contains_key(placeholder) {
                template.set(placeholder, value)?;
            }
        }
        Ok(())
    }

    /// Set a template reference as a value for a placeholder across all templates
    pub fn set_ref_global(&mut self, placeholder: &str, template_ref: TronRef) -> Result<()> {
        for template in &mut self.templates {
            if template.inner().placeholders.contains_key(placeholder) {
                template.set_ref(placeholder, template_ref.clone())?;
            }
        }
        Ok(())
    }

    /// Render all templates and combine them
    pub fn render_all(&self) -> Result<String> {
        let mut result = String::new();
        for template in &self.templates {
            result.push_str(&template.render()?);
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_composition() -> Result<()> {
        // Create a function template
        let function = TronTemplate::new("fn @[name]@() {\n    @[body]@\n}")?;
        let mut function_ref = TronRef::new(function);
        
        // Create a print template to insert into the function
        let print = TronTemplate::new("println!(\"@[message]@\");")?;
        let mut print_ref = TronRef::new(print);
        print_ref.set("message", "Hello from Tron!")?;
        
        // Compose the templates
        function_ref.set("name", "greet")?;
        function_ref.set_ref("body", print_ref)?;
        
        let rendered = function_ref.render()?;
        assert!(rendered.contains("fn greet()"));
        assert!(rendered.contains("println!(\"Hello from Tron!\");"));
        
        Ok(())
    }

    #[test]
    fn test_nested_composition() -> Result<()> {
        let outer = TronTemplate::new("mod test {\n    @[function]@\n}")?;
        let mut outer_ref = TronRef::new(outer);
        
        let inner = TronTemplate::new("fn helper() {\n    @[body]@\n}")?;
        let mut inner_ref = TronRef::new(inner);
        
        let print = TronTemplate::new("println!(\"@[message]@\");")?;
        let mut print_ref = TronRef::new(print);
        print_ref.set("message", "Nested template")?;
        
        inner_ref.set_ref("body", print_ref)?;
        outer_ref.set_ref("function", inner_ref)?;
        
        let rendered = outer_ref.render()?;
        assert!(rendered.contains("mod test {"));
        assert!(rendered.contains("fn helper()"));
        assert!(rendered.contains("println!(\"Nested template\");"));
        
        Ok(())
    }
}