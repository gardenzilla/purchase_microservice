// SKU to CART
// SKU, Derived Product, Depreciated

use crate::purchase::*;
use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct PurchaseOld {
  pub id: Uuid,                             // Cart ID UUID?
  pub customer: Option<Customer>,           // Only if there is any related one
  pub discount_percentage: Option<u32>,     // Applied discount
  pub items: Vec<Item>,                     // Cart items (All items: shopping list + unique)
  pub upl_info_objects: Vec<UplInfoObject>, // ALL UPL info objects
  pub total_net: u32,                       // Total cart net value in HUF
  pub total_vat: u32,                       // Total VAT
  pub total_gross: u32,                     // Total cart gross value in HUF
  pub document_kind: DocumentKind,          // Receipt or Invoice
  pub payment_kind: PaymentKind,            // cash, transfer, card
  pub payments: Vec<Payment>,               // Payment vector
  pub balance: i32,                         // Payment balance
  pub profit_net: i32,                      // Net profit
  pub owner_uid: u32,                       // Shop assistant UID
  pub store_id: Option<u32>,                // Now its stock ID
  pub date_completion: DateTime<Utc>,       // Completion date
  pub payment_duedate: DateTime<Utc>,       // Payment duedate
  pub restored: Option<Uuid>,               // Some(_) if its restored
  pub created_by: u32,                      // UID
  pub created_at: DateTime<Utc>,            // When cart created
}

impl From<PurchaseOld> for Purchase {
  fn from(f: PurchaseOld) -> Self {
    Self {
      id: f.id,
      customer: f.customer,
      discount_percentage: f.discount_percentage,
      items: f.items,
      upl_info_objects: f.upl_info_objects,
      total_net: f.total_net,
      total_vat: f.total_vat,
      total_gross: f.total_gross,
      document_kind: f.document_kind,
      payment_kind: f.payment_kind,
      payments: f.payments,
      payable: f.total_gross as i32,
      balance: f.balance,
      profit_net: f.profit_net,
      owner_uid: f.owner_uid,
      store_id: f.store_id,
      date_completion: f.date_completion,
      payment_duedate: f.payment_duedate,
      restored: f.restored,
      created_by: f.created_by,
      created_at: f.created_at,
    }
  }
}

impl Default for PurchaseOld {
  fn default() -> Self {
    Self {
      id: Uuid::default(),
      customer: None,
      discount_percentage: None,
      items: Vec::new(),
      upl_info_objects: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::default(),
      payment_kind: PaymentKind::default(),
      payments: Vec::new(),
      balance: 0,
      profit_net: 0,
      owner_uid: 0,
      store_id: None,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0),
      restored: None,
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl VecPackMember for PurchaseOld {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}
