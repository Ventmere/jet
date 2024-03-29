//! Implements Products API
//! [Jet Documentation](https://developer.jet.com/docs/overview)
//!

use super::client::{Client, Method};
use crate::error::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryFulfillmentNode {
  pub fulfillment_node_id: String,
  pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
  pub fulfillment_nodes: Vec<InventoryFulfillmentNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
  pub price: f32,
}

impl Client {
  pub fn update_inventory(&self, sku_id: &str, data: Inventory) -> Result<()> {
    self.request(
      Method::PUT,
      &format!("/merchant-skus/{}/inventory", sku_id),
      |req| {
        req.json(&data)
      },
    )
  }

  pub fn get_inventory(&self, sku_id: &str) -> Result<Inventory> {
    self.request(
      Method::GET,
      &format!("/merchant-skus/{}/inventory", sku_id),
      std::convert::identity,
    )
  }

  pub fn update_price(&self, sku_id: &str, data: Price) -> Result<()> {
    self.request(
      Method::PUT,
      &format!("/merchant-skus/{}/price", sku_id),
      |req| {
        req.json(&data)
      },
    )
  }

  pub fn get_price(&self, sku_id: &str) -> Result<Price> {
    self.request(
      Method::GET,
      &format!("/merchant-skus/{}/inventory", sku_id),
      std::convert::identity,
    )
  }
}
