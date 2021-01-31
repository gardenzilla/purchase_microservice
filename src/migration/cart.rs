// SKU to CART
// SKU, Derived Product, Depreciated

use crate::cart::*;
use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct CartOld {
  pub ancestor: Option<Uuid>,           // If this is a restored Cart
  pub id: Uuid,                         // Cart ID UUID?
  pub customer: Option<Customer>,       // Only if there is any related one
  pub discount_percentage: Option<u32>, // Discount in percentage 20% => 20
  pub shopping_list: Vec<ListItem>,     // Shopping list
  pub upls_sku: Vec<UplInfoObject>,     // UPLs that are healty
  pub upls_unique: Vec<UplInfoObject>,  // Upls that are depreciated or opened
  pub total_net: u32,                   // Total cart net value in HUF
  pub total_vat: u32,                   // Total VAT
  pub total_gross: u32,                 // Total cart gross value in HUF
  pub document_kind: DocumentKind,      // Receipt or Invoice
  pub payment_kind: PaymentKind,        // cash, transfer, card
  pub payments: Vec<Payment>,           // Payment vector
  pub owner_uid: u32,                   // Shop assistant UID
  pub store_id: Option<u32>,            // Now its stock ID
  pub date_completion: DateTime<Utc>,   // Invoice Completion date
  pub payment_duedate: DateTime<Utc>,   // Invoice Payment duedate
  pub created_by: u32,                  // UID
  pub created_at: DateTime<Utc>,        // When cart created
}

impl Default for CartOld {
  fn default() -> Self {
    Self {
      ancestor: None,
      id: Uuid::default(),
      customer: None,
      discount_percentage: None,
      shopping_list: Vec::new(),
      upls_sku: Vec::new(),
      upls_unique: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::default(),
      payment_kind: PaymentKind::default(),
      payments: Vec::default(),
      owner_uid: 0,
      store_id: None,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0),
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl VecPackMember for CartOld {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}

impl From<CartOld> for Cart {
  fn from(f: CartOld) -> Self {
    Self {
      ancestor: f.ancestor,
      id: f.id,
      customer: f.customer,
      discount_percentage: f.discount_percentage,
      shopping_list: f.shopping_list,
      upls_sku: f.upls_sku,
      upls_unique: f.upls_unique,
      total_net: f.total_net,
      total_vat: f.total_vat,
      total_gross: f.total_gross,
      document_kind: f.document_kind,
      payment_kind: f.payment_kind,
      payments: f.payments,
      payable: f.total_gross as i32,
      owner_uid: f.owner_uid,
      store_id: f.store_id,
      date_completion: f.date_completion,
      payment_duedate: f.payment_duedate,
      created_by: f.created_by,
      created_at: f.created_at,
    }
  }
}
