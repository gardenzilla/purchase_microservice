// SKU to CART
// SKU, Derived Product, Depreciated

use chrono::prelude::*;
use packman::VecPackMember;
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
  // Try to remove SKU
  fn remove_sku(&mut self, sku: u32) -> Result<&Self, String>;
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
  /// Get cart current payment balance
  fn get_balance(&self) -> i32;
  /// Get cart current profit
  fn get_profit_net(&self) -> i32;
  /// Close cart.sum::<i32>()
  /// If
  ///   SKU / UPL ok
  ///   Payment OK (Cash / Card and Payment OK)
  fn close_cart(&mut self) -> Result<&Self, String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cart {
  pub ancestor: Option<Uuid>,               // If this is a restored Cart
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
  pub created_by: u32,                      // UID
  pub created_at: DateTime<Utc>,            // When cart created
}

impl Default for Cart {
  fn default() -> Self {
    Self {
      ancestor: None,
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
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl VecPackMember for Cart {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}

impl CartMethods for Cart {
  fn new(owner_uid: u32, store_id: Option<u32>, created_by: u32) -> Self {
    Self {
      ancestor: None,
      id: Uuid::new_v4(),
      customer: None,
      items: Vec::new(),
      upl_info_objects: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::Receipt,
      payment_kind: PaymentKind::Cash,
      payments: Vec::new(),
      owner_uid: owner_uid,
      store_id: store_id,
      created_by: created_by,
      created_at: Utc::now(),
    }
  }

  fn add_customer(&mut self, customer: Option<Customer>) -> &Self {
    self.customer = customer;
    self
  }

  fn add_sku(
    &mut self,
    _sku: u32,
    piece: u32,
    name: String,
    vat: String,
    unit_retail_price_net: u32,
    unit_retail_price_gross: u32,
  ) -> &Self {
    // Check if the required SKU already in the cart items
    for item in &mut self.items {
      // If item is SKU
      if let ItemKind::Sku { sku } = item.kind {
        // If sku is the needed one
        if sku == _sku {
          // Set item piece to the new value
          // Overwrite it
          item.set_new_piece(piece);
          // Return self ref
          return self;
        }
      }
    }

    // We havent found SKU in the list yet
    // lets add it
    self.items.push(Item::new(
      // Here we can only create ItemKind::Sku {_}
      ItemKind::Sku { sku: _sku },
      name,
      piece,
      unit_retail_price_net,
      vat,
      unit_retail_price_gross,
    ));

    // Return self ref
    self
  }

  fn remove_sku(&mut self, _sku: u32) -> Result<&Self, String> {
    // Try to find SKU
    let sku_position = self
      .items
      .iter()
      .position(|i| match i.kind {
        ItemKind::Sku { sku } => sku == _sku,
        _ => false,
      })
      .ok_or("A kért SKU nem szerepel a kosárban, vagy bontott, vagy egyedi termék".to_string())?;

    // Check if SKU empty
    let sku_is_empty = match self.items.get(sku_position) {
      Some(s) => s.upl_ids.len() == 0,
      None => false,
    };

    // Remove SKU if empty
    if sku_is_empty {
      self.items.remove(sku_position);
    }

    // Otherwise return Error
    Err("A kért SKU nem üres! Törölj belőle minden UPL-t!".to_string())
  }

  fn add_upl(&mut self, upl: UplInfoObject) -> Result<&Self, String> {
    // Check if we already have this UPL in UPL Info Objects
    match self
      .upl_info_objects
      .iter()
      .find(|u| u.upl_id == upl.upl_id)
    {
      Some(_) => return Err("A kért UPL már szerepel a kosárban!".to_string()),
      None => (),
    }

    // Add UPL to Upl Info Objects
    self.upl_info_objects.push(upl.clone());

    // Check if new UPL is a unique one
    // If yes, then add it as a new unique item
    // and return
    match &upl.kind {
      // If its a SKU but depreciated, then its a unique one: DepreciatedSku
      UplKind::Sku { sku: _, piece } => {
        if upl.depreciated {
          self.items.push(Item::new(
            ItemKind::SkuDepreciated {
              upl_id: upl.upl_id.clone(),
            },
            upl.name.clone(),
            *piece,
            upl.retail_net_price,
            upl.vat.clone(),
            upl.retail_gross_price,
          ));
          // Return self ref
          return Ok(self);
        }
      }
      // If its a derived product, its a unique one
      UplKind::DerivedProduct {
        product_id: _,
        amount,
      } => {
        self.items.push(Item::new(
          ItemKind::DerivedProduct {
            upl_id: upl.upl_id.clone(),
            amount: *amount,
            unit: upl.unit.clone(),
          },
          upl.name.clone(),
          1, // Its always 1 as its a unique one
          upl.retail_net_price,
          upl.vat.clone(),
          upl.retail_gross_price,
        ));
        // Return self ref
        return Ok(self);
      }
    }

    // If its a not depreciated SKU
    if let UplKind::Sku { sku, piece } = &upl.kind {
      let new_upl_sku = *sku;
      // If there is any suitable Sku Item in Items
      // to place the new UPL then we store its position here
      // Otherwise its none, so lets create a new unique item
      let sku_position: Option<usize> = self.items.iter().position(|i| match &i.kind {
        ItemKind::Sku { sku } => *sku == new_upl_sku,
        ItemKind::SkuDepreciated { upl_id: _ } => false,
        ItemKind::DerivedProduct {
          upl_id: _,
          amount: _,
          unit: _,
        } => false,
      });

      match sku_position {
        // If we already have a related Item then add it there
        Some(p) => match &mut self.items.get_mut(p) {
          Some(ritem) => {
            // Set new piece
            ritem.set_new_piece(ritem.piece + piece);
            // Be sure, the UPL ID is not in the related list
            ritem.upl_ids.retain(|uid| uid != &upl.upl_id);
            // Add upl_id as related id
            ritem.upl_ids.push(upl.upl_id.clone());
            // Return self ref
            return Ok(self);
          }
          None => {}
        },
        // Create a new Item
        None => {
          // At last add a new item if we are here
          self.items.push(Item::new(
            ItemKind::Sku { sku: new_upl_sku },
            upl.name,
            *piece,
            upl.retail_net_price,
            upl.vat.clone(),
            upl.retail_gross_price,
          ));
          // Return self ref
          return Ok(self);
        }
      }
    }

    // if we are here, then its an error
    // Roll back UPL in
    self.upl_info_objects.retain(|uo| &uo.upl_id != &upl.upl_id);

    // and return an error
    Err("Kritikus hiba! A kért UPL nem tehető be a kosárba!".to_string())
  }

  fn remove_upl(&mut self, upl_id: String) -> Result<&Self, String> {
    // Try find UPL
    let upl_position = self
      .upl_info_objects
      .iter()
      .position(|u| u.upl_id == upl_id)
      .ok_or("A kért UPL nem szerepel a kosárban!".to_string())?;

    // Remove UPL from SKU
    // Iterate over all items
    for item in &mut self.items {
      // Try to find item which has the required UPL
      if item.upl_ids.contains(&upl_id) {
        // Remove UPL ID from upl_ids
        item.upl_ids.retain(|i| i != &upl_id);
      }
    }

    // Remove UPL from UPL info object vector
    self.upl_info_objects.remove(upl_position);

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
    self
      .items
      .iter()
      .map(|i| i.total_retail_price_gross)
      .sum::<u32>() as i32
      - self.get_payment_total()
  }

  fn get_profit_net(&self) -> i32 {
    self
      .upl_info_objects
      .iter()
      .map(|u| u.retail_net_price as i32 - u.procurement_net_price as i32)
      .sum::<i32>()
  }

  fn close_cart(&mut self) -> Result<&Self, String> {
    // Check if we can close
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
      PaymentKind::Transfer { payment_duedate: _ } => match self.document_kind {
        DocumentKind::Receipt => {
          return Err("A kosár nem zárható le, átutalás esetén kötelező számlát kérni!".to_string())
        }
        DocumentKind::Invoice => (),
      },
    }

    if self.items.iter().map(|i| i.upl_ids.len()).sum::<usize>() != self.upl_info_objects.len() {
      return Err("Lehetetlen hiba! A kosár nem zárható le! A kosárhoz rendelt UPL-ek száma eltérnek a tételekhez rendelt UPL-ek számától.".to_string());
    }

    Ok(self)
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

impl Item {
  pub fn new(
    kind: ItemKind,
    name: String,
    piece: u32,
    retail_price_net: u32,
    vat: String,
    retail_price_gross: u32,
  ) -> Self {
    Self {
      kind,
      name,
      piece,
      retail_price_net,
      vat,
      retail_price_gross,
      total_retail_price_net: retail_price_net * piece,
      total_retail_price_gross: retail_price_gross * piece,
      upl_ids: Vec::new(),
    }
  }
  pub fn set_new_piece(&mut self, new_piece: u32) -> &Self {
    // Set new piece
    self.piece = new_piece;
    // Reset total net
    self.total_retail_price_net = self.piece * self.retail_price_net;
    // Reset total gross
    self.total_retail_price_gross = self.piece * self.retail_price_gross;
    self
  }
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
