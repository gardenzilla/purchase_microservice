// SKU to CART
// SKU, Derived Product, Depreciated

use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Purchase {
  pub id: Uuid,                             // Cart ID UUID?
  pub customer: Option<Customer>,           // Only if there is any related one
  pub items: Vec<Item>,                     // Cart items
  pub upl_info_objects: Vec<UplInfoObject>, // Related UPL info objects
  pub total_net: u32,                       // Total cart net value in HUF
  pub total_vat: u32,                       // Total VAT
  pub total_gross: u32,                     // Total cart gross value in HUF
  pub document_kind: DocumentKind,          // Receipt or Invoice
  pub payment_kind: PaymentKind,            // cash, transfer, card
  pub payments: Vec<Payment>,               // Payment vector
  pub owner_uid: u32,                       // Shop assistant UID
  pub store_id: Option<u32>,                // Now its stock ID
  pub restored: Option<Uuid>,               // Some(_) if its restored
  pub created_by: u32,                      // UID
  pub created_at: DateTime<Utc>,            // When cart created
}

impl Default for Purchase {
  fn default() -> Self {
    Self {
      id: Uuid::default(),
      customer: None,
      items: Vec::new(),
      upl_info_objects: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::default(),
      payment_kind: PaymentKind::default(),
      payments: Vec::new(),
      owner_uid: 0,
      store_id: None,
      restored: None,
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl VecPackMember for Purchase {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DocumentKind {
  Receipt,
  Invoice,
}

impl Default for DocumentKind {
  fn default() -> Self {
    Self::Receipt
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Customer {
  pub id: u32,
  pub name: String,
  pub address: String,
  pub tax_number: String,
}

impl Default for Customer {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::default(),
      address: String::default(),
      tax_number: String::default(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PaymentKind {
  Cash,
  Card,
  Transfer { payment_duedate: DateTime<Utc> },
}

impl Default for PaymentKind {
  fn default() -> Self {
    Self::Cash
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Payment {
  pub payment_id: String,
  pub amount: i32,
}

impl Default for Payment {
  fn default() -> Self {
    Self {
      payment_id: String::default(),
      amount: 0,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
  pub kind: ItemKind,
  pub name: String,
  pub piece: u32,
  pub retail_price_net: u32,
  pub vat: String,
  pub retail_price_gross: u32,
  pub total_retail_price_net: u32,
  pub total_retail_price_gross: u32,
  pub upl_ids: Vec<String>,
}

impl Default for Item {
  fn default() -> Self {
    Self {
      kind: ItemKind::default(),
      name: String::default(),
      piece: 0,
      retail_price_net: 0,
      vat: String::default(),
      retail_price_gross: 0,
      total_retail_price_net: 0,
      total_retail_price_gross: 0,
      upl_ids: Vec::new(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ItemKind {
  // Sku or BulkSku
  Sku {
    sku: u32,
  },
  // Depreciated SKU or BulkSku
  SkuDepreciated {
    upl_id: String,
  },
  // OpenedSku or Derived Product
  DerivedProduct {
    upl_id: String,
    amount: u32,
    unit: String,
  },
  // Depreciated OpenedSku or Derived Product cannot add to cart
  // for now
}

impl Default for ItemKind {
  fn default() -> Self {
    Self::Sku { sku: 0 }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UplInfoObject {
  pub upl_id: String,
  pub kind: UplKind,
  pub name: String,
  pub unit: String,
  pub retail_net_price: u32,
  pub vat: String,
  pub retail_gross_price: u32,
  pub procurement_net_price: u32,
  pub best_before: DateTime<Utc>,
  pub depreciated: bool,
}

impl Default for UplInfoObject {
  fn default() -> Self {
    Self {
      upl_id: String::default(),
      kind: UplKind::default(),
      name: String::default(),
      unit: String::default(),
      retail_net_price: 0,
      vat: String::default(),
      retail_gross_price: 0,
      procurement_net_price: 0,
      best_before: Utc::now(),
      depreciated: false,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum UplKind {
  // Sku or BulkSku
  Sku { sku: u32, piece: u32 },
  // Opened Product or Derived Product
  DerivedProduct { product_id: u32, amount: u32 },
}

impl Default for UplKind {
  fn default() -> Self {
    Self::Sku { sku: 0, piece: 0 }
  }
}