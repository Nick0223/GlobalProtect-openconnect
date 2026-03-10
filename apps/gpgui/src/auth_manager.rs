use std::process::Command;
use std::io::{BufRead, BufReader};
use tokio::process::Command as TokioCommand;

pub struct AuthManager;

impl AuthManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn authenticate(&self, portal_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Build gpauth command
        let mut cmd = TokioCommand::new("gpauth");
        cmd.arg("--portal").arg(portal_url);
        cmd.arg("--browser").arg("default");
        
        // Execute gpauth and capture output
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(format!("gpauth failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        
        // Parse cookie from output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let cookie = self.parse_cookie_from_output(&output_str)?;
        
        Ok(cookie)
    }
    
    fn parse_cookie_from_output(&self, output: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Look for cookie in the output
        // The actual format depends on gpauth implementation
        for line in output.lines() {
            if line.starts_with("COOKIE=") {
                return Ok(line[7..].to_string());
            }
        }
        
        // If no explicit COOKIE= found, assume the entire output is the cookie
        if !output.trim().is_empty() {
            return Ok(output.trim().to_string());
        }
        
        Err("No cookie found in gpauth output".into())
    }
}