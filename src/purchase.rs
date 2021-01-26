// SKU to CART
// SKU, Derived Product, Depreciated

use crate::vat::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Cart {
  pub id: Uuid,                          // Cart ID UUID?
  pub customer: Option<Customer>,        // Only if there is any related one
  pub discount_percentage: Option<u32>,  // Read from Customer data
  pub items: Vec<ItemKind>,              // Cart items
  pub discount_items: Vec<DiscountItem>, // Generated discount items in HUF by VAT
  pub total_net: i32,                    // Total cart net value in HUF
  pub total_vat: Vec<VatItem>,           // Total VAT vector Vec<(VAT, u32)>
  pub total_gross: i32,                  // Total cart gross value in HUF
  pub need_invoice: bool,                // Invoice is generating if true
  pub invoice_id: Option<u32>,           // Invoice internal ID
  pub payment_amount: i32,               // Total payable amount (rounded to 0|5 if Cash)
  pub payment_kind: PaymentKind,         // cash, transfer, card
  pub payment_deadline: DateTime<Utc>,   // Payment deadline
  pub payments: Vec<Payment>,            // Payments vector (Cash ID + amount)
  pub owner_uid: u32,                    // Shop assistant UID
  pub store_id: u32,                     // Now its stock ID
  pub closed: bool,                      // todo? do we need this? or convert cart to purchase?
  pub created_by: u32,                   // UID
  pub created_at: DateTime<Utc>,         // When cart created
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Customer {
  pub id: u32,
  pub name: String,
  pub address: String,
  pub tax_number: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DiscountItem {
  discount_value_net: i32,
  vat: VAT,
  discount_value_gross: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VatItem {
  vat: VAT,
  vat_value: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PaymentKind {
  Cash,
  Card,
  Transfer,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Payment {
  cash_id: String,
  amount: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ItemKind {
  Sku {
    sku: u32,
    name: String,
    unit: String,
    quantity: String,
    retail_price_net: u32,
    retail_price_gross: u32,
    piece: u32,
    upls: Vec<UplSummary>,
  },
  SkuDepreciated {
    upl_id: String,
    name: String,
    unit: String,
    quantity: String,
    retail_price_net: u32,
    retail_price_gross: u32,
    upl: UplSummary,
  },
  // OpenedSku or Derived Product
  DerivedProduct {
    pid: u32,
    amount: u32,
    name: String,
    unit: String,
    quantity: String,
    retail_price_net: u32,
    retail_price_gross: u32,
    upls: Vec<UplSummary>,
  },
  // Depreciated OpenedSku or Derived Product cannot add to cart
  // for now
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UplSummary {
  upl_id: String,
  best_before: String,
  depreciated: bool,
  depreciated_net_price: Option<u32>,
  healthy: bool,
}

pub struct Purchase {
  pub id: Uuid,          // Cart ID
  pub payments: Vec<()>, //
  pub balance: i32,      //
}
