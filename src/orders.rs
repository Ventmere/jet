//! Implements Orders API
//! [Jet Documentation](https://developer.jet.com/docs/order-status)
//!

use super::client::{Client, Method};
use chrono::{DateTime, Utc};
use error::*;
use utils::serialize_datetime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderStatus {
  /// 'created' - The order has just been placed. Jet.com allows a half hour for fraud check and customer cancellation. We ask that retailers NOT fulfill orders that are created.
  #[serde(rename = "created")]
  Created,

  /// 'ready' - The order is ready to be fulfilled by the retailer
  #[serde(rename = "ready")]
  Ready,

  /// 'acknowledged' - The order has been accepted by the retailer and is awaiting fulfillment
  #[serde(rename = "acknowledged")]
  Acknowledged,

  /// 'inprogress' - The order is partially shipped
  #[serde(rename = "inprogress")]
  Inprogress,

  /// 'complete' - The order is completely shipped or cancelled. All units have been accounted for
  #[serde(rename = "complete")]
  Complete,
}

/// Shipping details about the order
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetail {
  pub request_shipping_carrier: Option<String>,
  pub request_shipping_method: String,
  pub request_service_level: String,
  pub request_ship_by: DateTime<Utc>,
  pub request_delivery_by: DateTime<Utc>,
}

/// Information about the buyer
#[derive(Debug, Serialize, Deserialize)]
pub struct Buyer {
  pub name: String,
  pub phone_number: String,
}

/// Information about the buyer
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
  pub address1: String,
  pub address2: Option<String>,
  pub city: String,
  pub state: String,
  pub zip_code: String,
}

/// Information about who and where the order will be shipped to
#[derive(Debug, Serialize, Deserialize)]
pub struct ShippingTo {
  pub recipient: Buyer,
  pub address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
  pub base_price: f32,
  pub item_tax: Option<f32>,
  pub item_shipping_cost: f32,
  pub item_shipping_tax: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeAdjustment {
  pub adjustment_name: String,
  pub adjustment_type: String,
  pub commission_id: String,
  pub value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderTotals {
  pub item_price: Option<Price>,
  pub item_fees: Option<f32>,
  pub fee_adjustments: Option<Vec<FeeAdjustment>>,
  pub regulatory_fees: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItem {
  pub order_item_id: String,
  pub alt_order_item_id: Option<String>,
  pub merchant_sku: String,
  pub product_title: String,
  pub request_order_quantity: i32,
  pub adjustment_reason: Option<String>,
  pub item_tax_code: Option<String>,
  pub url: String,
  pub price_adjustment: Option<f32>,
  pub item_fees: Option<f32>,
  pub fee_adjustments: Option<Vec<FeeAdjustment>>,
  // pub tax_info: Tax,
  pub regulatory_fees: Option<f32>,
  pub item_price: Price,

  /// When an order moves from "ready" to "acknowledged"
  pub order_item_acknowledgement_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipmentItem {
  pub shipment_item_id: Option<String>,
  pub alt_shipment_item_id: Option<String>,
  pub merchant_sku: String,
  pub response_shipment_sku_quantity: i32,
  pub response_shipment_cancel_qty: Option<i32>,
  #[serde(rename = "RMA_number")]
  pub rma_number: Option<String>,
  pub days_to_return: Option<i32>,
  pub return_location: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shipment {
  pub shipment_id: String,
  pub alt_shipment_id: Option<String>,
  pub shipment_tracking_number: Option<String>,
  pub response_shipment_date: Option<DateTime<Utc>>,
  pub response_shipment_method: Option<String>,
  pub expected_delivery_date: Option<DateTime<Utc>>,
  pub ship_from_zip_code: Option<String>,
  pub carrier: String,
  pub carrier_pick_up_date: Option<DateTime<Utc>>,
  pub shipment_items: Vec<ShipmentItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
  /// Jet's unique ID for a given merchant order.
  pub merchant_order_id: String,
  /// Jet's human readable order ID number that may have a small chance of collision overtime.
  pub reference_order_id: String,
  pub customer_reference_order_id: String,
  /// The fulfillment node that the order should be shipped from.
  pub fulfillment_node: String,
  /// Optional Merchant supplied order ID.If an alt_order_id has been associated with the merchant_order_id via the order accept message this will be passed as well.
  pub alt_order_id: Option<String>,
  /// The email hash assigned by Jet to be used as the customer email address
  pub hash_email: String,
  /// Current status of the order
  pub status: OrderStatus,
  /// Must be one of the following values:
  /// - exception - too many units cancelled
  /// - exception - jet manual canceled to complete state
  /// - exception - too many units shipped
  /// - exception - order rejected
  /// - resolved
  pub exception_state: Option<String>,
  /// The date the merchant order was placed.
  pub order_placed_date: DateTime<Utc>,
  /// Shipping details about the order
  pub order_detail: OrderDetail,
  /// Information about the buyer
  pub buyer: Buyer,
  /// Information about who and where the order will be shipped to
  pub shipping_to: ShippingTo,
  pub order_totals: OrderTotals,
  pub order_items: Vec<OrderItem>,

  // When an order moves from "created" to "ready"
  pub order_ready_date: Option<DateTime<Utc>>,
  pub has_shipments: bool,

  // When an order moves from "ready" to "acknowledged", the following fields are added
  pub order_acknowledge_date: Option<DateTime<Utc>>,
  /// Status to let Jet know whether you accept or reject the order.
  /// Errors that occur at the item level should be given the status
  // 'rejected - item level error'. This is returned in the order acknowledgement message.
  pub acknowledgement_status: Option<String>,

  // The following fields are provided by the merchant through the shipped message.
  // If multiple shipped messages are sent, shipment objects will be aggregated into the same shipments array
  pub shipments: Option<Vec<Shipment>>,
}

#[derive(Debug, Deserialize)]
pub struct GetOrdersResponse {
  pub order_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AcknowledgeOrderItem {
  /// Merchant defined fulfillable or nonfulfillable skus within the order.
  /// Must be one of the following values:
  /// - nonfulfillable - invalid merchant SKU
  /// - nonfulfillable - no inventory
  /// - fulfillable
  pub order_item_acknowledgement_status: &'static str,
  /// Jet's unique identifier for an item in a merchant order.
  pub order_item_id: String,
  /// Optional seller-supplied ID for an item in an order.
  /// If this value is specified with the Jet's order_item_id,
  /// Jet will map the two IDs and you can then use your own
  /// order item ID for subsequent feeds relating to that order item.
  pub alt_order_item_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AcknowledgeOrder {
  /// Status to let Jet know whether you accept or reject the order.
  /// Must be one of the following values:
  /// - rejected - other
  /// - rejected - fraud
  /// - rejected - item level error
  /// - rejected - ship from location not available
  /// - rejected - shipping method not supported
  /// - rejected - unfulfillable address
  /// - accepted
  pub acknowledgement_status: &'static str,
  pub alt_order_id: Option<String>,
  pub order_items: Vec<AcknowledgeOrderItem>,
}

#[derive(Debug, Serialize)]
pub struct ShipOrderShipmentItem {
  pub merchant_sku: String,
  pub response_shipment_sku_quantity: i32,
  pub days_to_return: i32,
}

#[derive(Debug, Serialize)]
pub struct ShipOrderShipment {
  pub carrier: String,
  pub shipment_tracking_number: Option<String>,
  pub shipment_items: Vec<ShipOrderShipmentItem>,
  #[serde(serialize_with = "serialize_datetime")]
  pub response_shipment_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ShipOrder {
  pub alt_order_id: Option<String>,
  pub shipments: Vec<ShipOrderShipment>,
}

impl Client {
  pub fn get_orders(&self, status: OrderStatus) -> Result<GetOrdersResponse> {
    self.request(
      Method::Get,
      &format!(
        "/orders/{}",
        match status {
          OrderStatus::Created => "created",
          OrderStatus::Ready => "ready",
          OrderStatus::Acknowledged => "acknowledged",
          OrderStatus::Inprogress => "inprogress",
          OrderStatus::Complete => "complete",
        }
      ),
      |_| Ok(()),
    )
  }

  pub fn get_order_detail(&self, order_url: &str) -> Result<Order> {
    self.request(Method::Get, order_url, |_| Ok(()))
  }

  pub fn acknowledge_order(&self, order_id: &str, ack: &AcknowledgeOrder) -> Result<()> {
    self.request_no_content(
      Method::Put,
      &format!("/orders/{}/acknowledge", order_id),
      |req| {
        req.json(ack);
        Ok(())
      },
    )
  }

  pub fn ship_order(&self, order_id: &str, ship: &ShipOrder) -> Result<()> {
    self.request_no_content(
      Method::Put,
      &format!("/orders/{}/shipped", order_id),
      |req| {
        req.json(ship);
        Ok(())
      },
    )
  }
}

#[test]
fn test_get_orders() {
  use client::get_test_client;
  let client = get_test_client();
  println!("{:#?}", client.get_orders(OrderStatus::Ready).unwrap());
}

#[test]
fn test_get_order_detail() {
  use client::get_test_client;
  let client = get_test_client();
  println!(
    "{:#?}",
    client
      .get_order_detail("/orders/withoutShipmentDetail/2ab4c8b414124f0fa04072d615ec0610")
      .unwrap()
  );
}

#[test]
fn test_acknowledge_order() {
  use client::get_test_client;
  let client = get_test_client();
  println!(
    "{:#?}",
    client
      .acknowledge_order(
        "2ab4c8b414124f0fa04072d615ec0610",
        &AcknowledgeOrder {
          acknowledgement_status: "accepted",
          alt_order_id: None,
          order_items: vec![AcknowledgeOrderItem {
            order_item_acknowledgement_status: "fulfillable",
            order_item_id: "2906d22b212d4745ab9986b80b1ad2af".to_owned(),
            alt_order_item_id: None,
          }],
        }
      )
      .unwrap()
  );
}

#[test]
fn test_ship_order() {
  use client::get_test_client;
  let client = get_test_client();
  client
    .ship_order(
      "2ab4c8b414124f0fa04072d615ec0610",
      &ShipOrder {
        alt_order_id: None,
        shipments: vec![ShipOrderShipment {
          carrier: "UPS".to_owned(),
          shipment_tracking_number: Some("1Z12342452342".to_owned()),
          shipment_items: vec![ShipOrderShipmentItem {
            merchant_sku: "test_product".to_owned(),
            response_shipment_sku_quantity: 1,
            days_to_return: 30,
          }],
          response_shipment_date: Utc::now(),
        }],
      },
    )
    .unwrap()
}

#[test]
fn test_unserialize_orders() {
  use serde_json::{self, Value};
  use std::fs::File;
  use std::io::ErrorKind;
  let f = match File::open("test_data/orders.json") {
    Ok(f) => f,
    Err(e) => match e.kind() {
      ErrorKind::NotFound => return,
      e => panic!("read order data error: {:?}", e),
    },
  };

  let values: Vec<Value> = serde_json::from_reader(f).unwrap();
  for value in values {
    let pretty = serde_json::to_string_pretty(&value).unwrap();
    match serde_json::from_str::<Order>(&pretty) {
      Ok(_) => {}
      Err(err) => panic!("{}\n{}", err, pretty),
    }
  }
}

#[test]
fn test_download_all_orders() {
  use client::get_test_client;
  use serde_json;
  let client = get_test_client();

  let mut orders = vec![];

  for status in vec![
    OrderStatus::Created,
    OrderStatus::Ready,
    OrderStatus::Acknowledged,
    OrderStatus::Inprogress,
    OrderStatus::Complete,
  ] {
    println!("loading orders: {:?}", status);
    let res = client.get_orders(status).unwrap();
    let len = res.order_urls.len();
    println!("- found {}", len);

    for (i, url) in res.order_urls.into_iter().enumerate() {
      println!("- downloading {} of {}...", i + 1, len);
      let order = client.get_order_detail(&url).unwrap();
      orders.push(order);
    }
  }

  {
    use std::fs::File;
    println!("writing to file...");
    let f = File::create("target/orders.json").unwrap();
    serde_json::to_writer_pretty(f, &orders).unwrap();
  }
}
