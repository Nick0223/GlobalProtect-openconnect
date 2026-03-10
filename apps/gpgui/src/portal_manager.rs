use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Portal {
    pub id: String,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub is_default: bool,
}

impl Portal {
    pub fn new(name: String, url: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            name,
            url,
            username: None,
            is_default: false,
        }
    }
}

pub struct PortalManager {
    portals: Vec<Portal>,
    portal_map: HashMap<String, usize>,
}

impl PortalManager {
    pub fn new() -> Self {
        Self {
            portals: Vec::new(),
            portal_map: HashMap::new(),
        }
    }
    
    pub fn get_portals(&self) -> &Vec<Portal> {
        &self.portals
    }
    
    pub async fn add_portal(&mut self, portal: Portal) -> Result<(), Box<dyn std::error::Error>> {
        self.portal_map.insert(portal.id.clone(), self.portals.len());
        self.portals.push(portal);
        Ok(())
    }
    
    pub async fn remove_portal(&mut self, portal_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.portal_map.get(portal_id) {
            self.portals.remove(*index);
            // Rebuild the map
            self.portal_map.clear();
            for (i, portal) in self.portals.iter().enumerate() {
                self.portal_map.insert(portal.id.clone(), i);
            }
        }
        Ok(())
    }
    
    pub async fn update_portal(&mut self, portal: Portal) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.portal_map.get(&portal.id) {
            self.portals[*index] = portal;
        }
        Ok(())
    }
    
    pub fn get_default_portal(&self) -> Option<&Portal> {
        self.portals.iter().find(|p| p.is_default)
    }
}