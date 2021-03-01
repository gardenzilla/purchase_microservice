// SKU to CART
// SKU, Derived Product, Depreciated

use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait PurchaseExt {
  /// Set related invoice ID
  fn set_invoice_id(&mut self, invoice_id: String) -> Result<&Self, String>;
  /// Set loyalty summary info
  /// given by the loyalty service
  fn set_loyalty_summary(
    &mut self,
    balance_opening: i32,
    burned_points: i32,
    earned_points: i32,
    balance_closing: i32,
  ) -> Result<&Self, String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Purchase {
  pub id: Uuid,                               // Cart ID UUID?
  pub customer: Option<Customer>,             // Only if there is any related one
  pub commitment: Option<Commitment>,         // Applied customer commitment
  pub commitment_discount_value: u32,         //
  pub loyalty_card: Option<LoyaltyCard>,      // Applied loyalty card
  pub items: Vec<Item>,                       // Cart items (All items: shopping list + unique)
  pub upl_info_objects: Vec<UplInfoObject>,   // ALL UPL info objects
  pub total_net: u32,                         // Total cart net value in HUF
  pub total_vat: u32,                         // Total VAT
  pub total_gross: u32,                       // Total cart gross value in HUF
  pub document_kind: DocumentKind,            // Receipt or Invoice
  pub payment_kind: PaymentKind,              // cash, transfer, card
  pub payments: Vec<Payment>,                 // Payment vector
  pub burned_points: Vec<LoyaltyTransaction>, // Burned payment points
  pub burned_loyalty_points: u32,             // Burned loyalty points total (gross)
  pub payable: i32,                           // Payable amount
  pub balance: i32,                           // Payment balance
  pub profit_net: i32,                        // Net profit
  pub owner_uid: u32,                         // Shop assistant UID
  pub store_id: Option<u32>,                  // Now its stock ID
  pub date_completion: DateTime<Utc>,         // Completion date
  pub payment_duedate: DateTime<Utc>,         // Payment duedate
  pub restored: Option<Uuid>,                 // Some(_) if its restored
  pub invoice: Option<String>,                // Invoice
  pub storno_invoice: Option<String>,         // Storno invoice
  pub created_by: u32,                        // UID
  pub created_at: DateTime<Utc>,              // When cart created
}

impl PurchaseExt for Purchase {
  fn set_invoice_id(&mut self, invoice_id: String) -> Result<&Self, String> {
    match self.invoice {
      Some(_) => Err("A vásárlás már rendelkezik számlával".to_string()),
      None => {
        self.invoice = Some(invoice_id);
        Ok(self)
      }
    }
  }

  fn set_loyalty_summary(
    &mut self,
    balance_opening: i32,
    burned_points: i32,
    earned_points: i32,
    balance_closing: i32,
  ) -> Result<&Self, String> {
    match &mut self.loyalty_card {
      Some(loyalty) => {
        // Check if burned points are ok
        if self.burned_loyalty_points as i32 != burned_points {
          return Err(
            "A vásárláshoz rendelt felhasznált pontok összege nem egyezik meg az
          összefoglaló szerinti felhasznált pontok összegével."
              .to_string(),
          );
        }
        loyalty.balance_opening = balance_opening;
        loyalty.burned_points = burned_points;
        loyalty.earned_points = earned_points;
        loyalty.balance_closing = balance_closing;
        Ok(self)
      }
      None => Err(
        "A vásárláshoz nem tartozik törzsvásárlói kártya,
        így nem lehet hozzá adatokat adni."
          .to_string(),
      ),
    }
  }
}

impl Default for Purchase {
  fn default() -> Self {
    Self {
      id: Uuid::default(),
      customer: None,
      commitment: None,
      commitment_discount_value: 0,
      loyalty_card: None,
      items: Vec::new(),
      upl_info_objects: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::default(),
      payment_kind: PaymentKind::default(),
      payments: Vec::new(),
      burned_points: Vec::new(),
      burned_loyalty_points: 0,
      payable: 0,
      balance: 0,
      profit_net: 0,
      owner_uid: 0,
      store_id: None,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0),
      restored: None,
      invoice: None,
      storno_invoice: None,
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
  pub zip: String,
  pub location: String,
  pub street: String,
  pub tax_number: String,
}

impl Default for Customer {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::default(),
      zip: String::default(),
      location: String::default(),
      street: String::default(),
      tax_number: String::default(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PaymentKind {
  Cash,
  Card,
  Transfer,
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
  pub product_id: u32,
  pub name: String,
  pub piece: u32,
  pub retail_price_net: u32,
  pub vat: String,
  pub retail_price_gross: u32,
  pub total_retail_price_net: u32,
  pub total_retail_price_gross: u32,
}

impl Default for Item {
  fn default() -> Self {
    Self {
      kind: ItemKind::default(),
      product_id: 0,
      name: String::default(),
      piece: 0,
      retail_price_net: 0,
      vat: String::default(),
      retail_price_gross: 0,
      total_retail_price_net: 0,
      total_retail_price_gross: 0,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ItemKind {
  // Sku or BulkSku
  Sku,
  // Depreciated SKU or BulkSku
  SkuDepreciated,
  // OpenedSku or Derived Product
  DerivedProduct,
  // Depreciated OpenedSku or Derived Product cannot add to cart
  // for now
}

impl Default for ItemKind {
  fn default() -> Self {
    Self::Sku
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UplInfoObject {
  pub upl_id: String,
  pub kind: UplKind,
  pub name: String,
  pub retail_net_price: u32,
  pub vat: String,
  pub retail_gross_price: u32,
  pub procurement_net_price: u32,
  pub best_before: Option<DateTime<Utc>>,
  pub depreciated: bool,
}

impl Default for UplInfoObject {
  fn default() -> Self {
    Self {
      upl_id: String::default(),
      kind: UplKind::default(),
      name: String::default(),
      retail_net_price: 0,
      vat: String::default(),
      retail_gross_price: 0,
      procurement_net_price: 0,
      best_before: None,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Commitment {
  pub commitment_id: Uuid,
  pub commitment_percentage: u32,
}

impl Default for Commitment {
  fn default() -> Self {
    Self {
      commitment_id: Uuid::default(),
      commitment_percentage: 0,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LoyaltyLevel {
  L1,
  L2,
}

impl ToString for LoyaltyLevel {
  fn to_string(&self) -> String {
    match self {
      LoyaltyLevel::L1 => "L1".to_string(),
      LoyaltyLevel::L2 => "L2".to_string(),
    }
  }
}

impl Default for LoyaltyLevel {
  fn default() -> Self {
    Self::L1
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoyaltyCard {
  pub account_id: Uuid,    // Loyalty account ID
  pub card_id: String,     // Loyalty card ID
  pub level: LoyaltyLevel, // L1 | L2
  pub balance_opening: i32,
  pub burned_points: i32,
  pub earned_points: i32,
  pub balance_closing: i32,
}

impl Default for LoyaltyCard {
  fn default() -> Self {
    Self {
      account_id: Uuid::default(),
      card_id: String::default(),
      level: LoyaltyLevel::default(),
      balance_opening: 0,
      burned_points: 0,
      earned_points: 0,
      balance_closing: 0,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoyaltyTransaction {
  pub loyalty_account_id: Uuid,
  pub transaction_id: Uuid,
  pub burned_points: i32,
  pub created_at: DateTime<Utc>,
}

impl Default for LoyaltyTransaction {
  fn default() -> Self {
    Self {
      loyalty_account_id: Uuid::default(),
      transaction_id: Uuid::default(),
      burned_points: 0,
      created_at: Utc::now(),
    }
  }
}
