// SKU to CART
// SKU, Derived Product, Depreciated

use chrono::prelude::*;
use packman::{TryFrom, VecPackMember};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait CartMethods
where
  Self: Sized,
{
  /// Create new Cart
  fn new(owner_uid: u32, store_id: Option<u32>, created_by: u32) -> Self;
  /// Add Customer to a cart
  fn add_customer(&mut self, customer: Option<Customer>) -> &Self;
  /// Add SKU to cart; Normal sku
  fn add_sku(
    &mut self,
    sku: u32,
    piece: u32,
    name: String,
    vat: String,
    unit_retail_price_net: u32,
    unit_retail_price_gross: u32,
  ) -> &Self;
  /// Try to remove SKU
  fn remove_sku(&mut self, sku: u32) -> Result<&Self, String>;
  /// Try to update SKU piece in shopping list
  fn set_sku_piece(&mut self, sku: u32, piece: u32) -> Result<&Self, String>;
  /// Try to add UPL to cart
  fn add_upl(&mut self, upl: UplInfoObject) -> Result<&Self, String>;
  /// Try to remove UPL from cart
  fn remove_upl(&mut self, upl_id: String) -> Result<&Self, String>;
  /// Set if invoice need
  fn set_document(&mut self, document_kind: DocumentKind) -> &Self;
  /// Set payment kind
  fn set_payment(&mut self, payment_kind: PaymentKind) -> &Self;
  /// Get the payments total
  fn get_payment_total(&self) -> i32;
  /// Add payment to Cart
  fn add_payment(&mut self, payment: Payment) -> &Self;
  /// Set owner to cart
  fn set_owner(&mut self, owner_uid: u32) -> &Self;
  /// Set store id; where the cart physically located
  fn set_store_id(&mut self, store_id: Option<u32>) -> &Self;
  /// Get payable amount
  fn get_payable(&self) -> i32;
  /// Get cart current payment balance
  fn get_balance(&self) -> i32;
  /// Get cart current profit
  fn get_profit_net(&self) -> i32;
  // Recalculate totals
  fn calculate_totals(&mut self);
  /// Close cart.sum::<i32>()
  /// If
  ///   SKU / UPL ok
  ///   Payment OK (Cash / Card and Payment OK)
  fn close_cart(&mut self) -> Result<&Self, String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cart {
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
  pub payable: i32,                     // Payable amount
  pub owner_uid: u32,                   // Shop assistant UID
  pub store_id: Option<u32>,            // Now its stock ID
  pub date_completion: DateTime<Utc>,   // Invoice Completion date
  pub payment_duedate: DateTime<Utc>,   // Invoice Payment duedate
  pub created_by: u32,                  // UID
  pub created_at: DateTime<Utc>,        // When cart created
}

impl Default for Cart {
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
      payable: 0,
      owner_uid: 0,
      store_id: None,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0),
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListItem {
  pub sku: u32,
  pub name: String,
  pub piece: u32,
  pub vat: String,
  pub unit_price_net: u32,
  pub unit_price_vat: u32,
  pub unit_price_gross: u32,
  pub total_price_net: u32,
  pub total_price_vat: u32,
  pub total_price_gross: u32,
}

impl ListItem {
  fn new(
    sku: u32,
    name: String,
    piece: u32,
    vat: String,
    unit_price_net: u32,
    unit_price_gross: u32,
  ) -> Self {
    let total_net = unit_price_net * piece;
    let total_gross = unit_price_gross * piece;
    Self {
      sku,
      name,
      piece,
      vat,
      unit_price_net,
      unit_price_vat: unit_price_gross - unit_price_net,
      unit_price_gross,
      total_price_net: total_net,
      total_price_vat: total_gross - total_net,
      total_price_gross: total_gross,
    }
  }
  // Update piece by replacing it
  fn update_piece(&mut self, piece: u32) -> &Self {
    // Update piece
    self.piece = piece;
    // Update total net
    self.total_price_net = self.unit_price_net * piece;
    // Update total gross
    self.total_price_gross = self.unit_price_gross * piece;
    // Update total VAT
    self.total_price_vat = self.total_price_gross - self.total_price_net;
    // Return self ref
    self
  }
  // Update piece by adding new ones
  fn update_add_piece(&mut self, plus_piece: u32) -> &Self {
    self.update_piece(self.piece + plus_piece);
    self
  }
  fn replace(&mut self, new_item: ListItem) -> &Self {
    let _ = std::mem::replace(self, new_item);
    self
  }
}

impl Default for ListItem {
  fn default() -> Self {
    Self {
      sku: 0,
      name: String::new(),
      piece: 0,
      vat: String::new(),
      unit_price_net: 0,
      unit_price_vat: 0,
      unit_price_gross: 0,
      total_price_net: 0,
      total_price_vat: 0,
      total_price_gross: 0,
    }
  }
}

impl VecPackMember for Cart {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}

impl TryFrom for Cart {
  type TryFrom = crate::migration::cart::CartOld;
}

impl CartMethods for Cart {
  fn new(owner_uid: u32, store_id: Option<u32>, created_by: u32) -> Self {
    Self {
      ancestor: None,
      id: Uuid::new_v4(),
      customer: None,
      discount_percentage: None,
      shopping_list: Vec::default(),
      upls_sku: Vec::default(),
      upls_unique: Vec::default(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::Receipt,
      payment_kind: PaymentKind::Cash,
      payments: Vec::default(),
      payable: 0,
      owner_uid,
      store_id,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0),
      created_by,
      created_at: Utc::now(),
    }
  }

  fn add_customer(&mut self, customer: Option<Customer>) -> &Self {
    self.customer = customer;
    self
  }

  fn add_sku(
    &mut self,
    sku: u32,
    piece: u32,
    name: String,
    vat: String,
    unit_retail_price_net: u32,
    unit_retail_price_gross: u32,
  ) -> &Self {
    // Try to find sku in shopping list
    let pos = self.shopping_list.iter().position(|i| i.sku == sku);

    let new_sku = ListItem::new(
      sku,
      name,
      piece,
      vat,
      unit_retail_price_net,
      unit_retail_price_gross,
    );

    match pos {
      // If we found it, lets update it
      Some(p) => {
        if let Some(item) = self.shopping_list.get_mut(p) {
          item.replace(new_sku);
        }
      }
      // Othwise lets push it
      None => self.shopping_list.push(new_sku),
    }

    // Recalculate cart totals
    self.calculate_totals();

    self
  }

  fn remove_sku(&mut self, sku: u32) -> Result<&Self, String> {
    // Check if there is any sku in upls
    if self
      .upls_sku
      .iter()
      .filter(|u| {
        if let Some(_sku) = u.get_sku() {
          return _sku == sku;
        }
        false
      })
      .count()
      > 0
    {
      return Err("Az adott SKU-hoz még van UPL a kosárban!".to_string());
    }

    // Remove from shopping_list
    self.shopping_list.retain(|i| i.sku != sku);

    self.calculate_totals();

    Ok(self)
  }

  fn set_sku_piece(&mut self, sku: u32, piece: u32) -> Result<&Self, String> {
    for item in &mut self.shopping_list {
      if item.sku == sku {
        // Update sku piece if we found it
        item.update_piece(piece);
        // Recalculate the cart totals
        self.calculate_totals();
        return Ok(self);
      }
    }
    Err("A kért SKU nem szerepel a kosárban!".to_string())
  }

  fn add_upl(&mut self, upl: UplInfoObject) -> Result<&Self, String> {
    // Check if UPL is in the SKU upls
    // If yes, then return error
    if self
      .upls_sku
      .iter()
      .find(|u| u.upl_id == upl.upl_id)
      .is_some()
    {
      return Err("A kért UPL már a kosárban van!".to_string());
    }

    // Check if UPL is in the Unique upls
    // If yes, then return error
    if self
      .upls_unique
      .iter()
      .find(|u| u.upl_id == upl.upl_id)
      .is_some()
    {
      return Err("A kért UPL már a kosárban van!".to_string());
    }

    match upl.kind {
      UplKind::Sku { sku, piece } => {
        // Add to unique UPLs
        if upl.depreciated {
          self.upls_unique.push(upl.clone());
        } else {
          self.upls_sku.push(upl.clone());
        }
      }
      // Add to unique UPLs
      UplKind::DerivedProduct { product_id, amount } => self.upls_unique.push(upl.clone()),
    }

    // Add it as a SKU if a normal SKU
    if let UplKind::Sku { sku, piece } = upl.kind {
      if !upl.depreciated {
        for item in &mut self.shopping_list {
          if item.sku == sku {
            item.update_add_piece(piece);
            self.calculate_totals();
            return Ok(self);
          }
        }
        // Add it as a new SKU otherwise
        self.shopping_list.push(ListItem::new(
          sku,
          upl.name.clone(),
          piece,
          upl.vat.clone(),
          upl.retail_net_price,
          upl.retail_gross_price,
        ));
      }
    }

    self.calculate_totals();
    Ok(self)
  }

  fn remove_upl(&mut self, upl_id: String) -> Result<&Self, String> {
    // Remove from Upl Sku if its there
    self.upls_sku.retain(|u| u.upl_id != upl_id);
    // Remove from Upl Unique if its there
    self.upls_unique.retain(|u| u.upl_id != upl_id);
    // Recalculate the cart totals
    self.calculate_totals();
    // Return self as ref
    Ok(self)
  }

  fn set_document(&mut self, document_kind: DocumentKind) -> &Self {
    self.document_kind = document_kind;
    self
  }

  fn get_payment_total(&self) -> i32 {
    self.payments.iter().map(|p| p.amount).sum()
  }

  fn set_payment(&mut self, payment_kind: PaymentKind) -> &Self {
    // TODO! Maybe some validation before set new value?
    self.payment_kind = payment_kind;
    self
  }

  fn add_payment(&mut self, payment: Payment) -> &Self {
    self.payments.push(payment);
    self
  }

  fn set_owner(&mut self, owner_uid: u32) -> &Self {
    self.owner_uid = owner_uid;
    self
  }

  fn set_store_id(&mut self, store_id: Option<u32>) -> &Self {
    self.store_id = store_id;
    self
  }

  fn get_balance(&self) -> i32 {
    self.payable - self.get_payment_total()
  }

  fn get_payable(&self) -> i32 {
    self.payable
  }

  fn get_profit_net(&self) -> i32 {
    self.total_net as i32
      - (self
        .upls_sku
        .iter()
        .map(|u| u.procurement_net_price as i32)
        .sum::<i32>()
        + self
          .upls_unique
          .iter()
          .map(|u| u.procurement_net_price as i32)
          .sum::<i32>())
  }

  fn close_cart(&mut self) -> Result<&Self, String> {
    // Check items count match
    for item in &self.shopping_list {
      if item.piece
        != self
          .upls_sku
          .iter()
          .filter(|u| {
            if let UplKind::Sku { sku, piece } = u.kind {
              return sku == item.sku;
            }
            false
          })
          .map(|u| u.get_piece())
          .sum::<u32>()
      {
        return Err("Rendezd a kosarat! Minden SKU-hoz UPL-t kell rendelni!".to_string());
      }
    }
    // Check totals
    let mut _total_net = 0;
    let mut _total_vat = 0;
    let mut _total_gross = 0;
    for item in &self.shopping_list {
      _total_net += item.total_price_net;
      _total_vat += item.total_price_vat;
      _total_gross += item.total_price_gross;
    }
    for item in &self.upls_unique {
      _total_net += item.get_price_net();
      _total_vat += item.get_price_vat();
      _total_gross += item.get_price_gross();
    }
    if _total_net != self.total_net
      || _total_vat != self.total_vat
      || _total_gross != self.total_gross
    {
      return Err("A kosár záró összegei nem helyesek! Nem lehet lezárni!".to_string());
    }
    // Check payment
    match self.payment_kind {
      PaymentKind::Cash => {
        if self.get_balance() != 0 {
          return Err(
            "A kosár nem zárható le, készpénzes fizetés esetén rendezze a befizetést!".to_string(),
          );
        }
      }
      PaymentKind::Card => {
        if self.get_balance() != 0 {
          return Err(
            "A kosár nem zárható le, bankkártyás fizetés esetén rendezze a befizetést!".to_string(),
          );
        }
      }
      PaymentKind::Transfer => match self.document_kind {
        DocumentKind::Receipt => {
          return Err("A kosár nem zárható le, átutalás esetén kötelező számlát kérni!".to_string())
        }
        DocumentKind::Invoice => (),
      },
    }

    // Return self as ref
    // Important to remove this cart in higher level
    // and transform it to purchase
    Ok(self)
  }

  fn calculate_totals(&mut self) {
    // Set new total net
    self.total_net = self
      .shopping_list
      .iter()
      .map(|i| i.total_price_net)
      .sum::<u32>()
      + self
        .upls_unique
        .iter()
        .map(|u| u.get_price_net())
        .sum::<u32>();

    // Set new total gross
    self.total_gross = self
      .shopping_list
      .iter()
      .map(|i| i.total_price_gross)
      .sum::<u32>()
      + self
        .upls_unique
        .iter()
        .map(|u| u.get_price_gross())
        .sum::<u32>();

    // Set new total vat
    self.total_vat = self.total_gross - self.total_net;

    // Set payable
    self.payable = match self.payment_kind {
      PaymentKind::Cash => crate::rounding::round_huf(self.total_gross as i32),
      _ => self.total_gross as i32,
    }
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

impl UplInfoObject {
  /// Try to get UPL related SKU
  /// if there is any
  pub fn get_sku(&self) -> Option<u32> {
    match &self.kind {
      UplKind::Sku { sku, piece: _ } => Some(*sku),
      _ => None,
    }
  }
  /// Try to get UPL related product ID
  /// if there is any
  pub fn get_product_id(&self) -> Option<u32> {
    match &self.kind {
      UplKind::DerivedProduct {
        product_id,
        amount: _,
      } => Some(*product_id),
      _ => None,
    }
  }
  /// Get UPL price net
  pub fn get_price_net(&self) -> u32 {
    match self.kind {
      UplKind::Sku { sku, piece } => self.retail_net_price * piece,
      UplKind::DerivedProduct { product_id, amount } => self.retail_net_price,
    }
  }
  /// Get UPL price gross
  pub fn get_price_gross(&self) -> u32 {
    match self.kind {
      UplKind::Sku { sku, piece } => self.retail_gross_price * piece,
      UplKind::DerivedProduct { product_id, amount } => self.retail_gross_price,
    }
  }
  /// Get UPL price VAT
  pub fn get_price_vat(&self) -> u32 {
    self.get_price_gross() - self.get_price_net()
  }
  /// Get piece
  pub fn get_piece(&self) -> u32 {
    match self.kind {
      UplKind::Sku { sku: _, piece } => piece,
      UplKind::DerivedProduct {
        product_id: _,
        amount: _,
      } => 1,
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
