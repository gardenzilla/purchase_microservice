// SKU to CART
// SKU, Derived Product, Depreciated

use std::ops::Mul;

use chrono::{prelude::*, Duration};
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
    vat: VAT,
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
  /// Add loyalty card to the cart
  fn add_loyalty_card(
    &mut self,
    account_id: Uuid,
    card_id: String,
    loyalty_level: LoyaltyLevel,
  ) -> Result<&Self, String>;
  /// Try to remove loyalty card
  fn remove_loyalty_card(&mut self) -> Result<&Self, String>;
  /// Get burned loyalty points balance
  fn get_burned_points_balance(&self) -> u32;
  /// Burn points
  fn burn_points(
    &mut self,
    loyalty_account_id: Uuid,
    transaction_id: Uuid,
    points_to_burn: i32,
  ) -> Result<&Self, String>;
  /// Add commitment to cart
  fn add_commitment(
    &mut self,
    commitment_id: Uuid,
    discount_percentage: u32,
  ) -> Result<&Self, String>;
  /// Remove commitment from cart
  fn remove_commitment(&mut self) -> Result<&Self, String>;
  /// Get the given discount based on the commitment
  fn get_commitment_discount_value(&self) -> u32;
  fn get_items_total_net(&self) -> u32;
  fn get_items_total_gross(&self) -> u32;
  fn get_items_total_vat(&self) -> u32;
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum VAT {
  AAM,
  FAD,
  TAM,
  _5,
  _18,
  _27,
}

impl Default for VAT {
  fn default() -> Self {
    VAT::_27
  }
}

impl VAT {
  pub fn from_str(str: &str) -> Result<VAT, String> {
    match str {
      "AAM" => Ok(VAT::AAM),
      "aam" => Ok(VAT::AAM),
      "FAD" => Ok(VAT::FAD),
      "fad" => Ok(VAT::FAD),
      "TAM" => Ok(VAT::TAM),
      "tam" => Ok(VAT::TAM),
      "5" => Ok(VAT::_5),
      "18" => Ok(VAT::_18),
      "27" => Ok(VAT::_27),
      _ => Err("Nem megfelelő Áfa formátum! 5, 18, 27, AAM, TAM, FAD".into()),
    }
  }
}

impl ToString for VAT {
  fn to_string(&self) -> String {
    match self {
      VAT::AAM => "AAM".to_string(),
      VAT::FAD => "FAD".to_string(),
      VAT::TAM => "TAM".to_string(),
      VAT::_5 => "5".to_string(),
      VAT::_18 => "18".to_string(),
      VAT::_27 => "27".to_string(),
    }
  }
}

impl Mul<VAT> for u32 {
  type Output = u32;

  fn mul(self, rhs: VAT) -> Self::Output {
    let res = match rhs {
      VAT::AAM => self as f32 * 1.0,
      VAT::FAD => self as f32 * 1.0,
      VAT::TAM => self as f32 * 1.0,
      VAT::_5 => self as f32 * 1.05,
      VAT::_18 => self as f32 * 1.18,
      VAT::_27 => self as f32 * 1.27,
    };
    res.round() as u32
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cart {
  pub ancestor: Option<Uuid>,                 // If this is a restored Cart
  pub id: Uuid,                               // Cart ID UUID?
  pub customer: Option<Customer>,             // Only if there is any related one
  pub commitment: Option<Commitment>,         // Applied customer commitment
  pub commitment_discount_value: u32,         // Commitment value
  pub loyalty_card: Option<LoyaltyCard>,      // Applied loyalty card
  pub shopping_list: Vec<ListItem>,           // Shopping list
  pub upls_sku: Vec<UplInfoObject>,           // UPLs that are healty
  pub upls_unique: Vec<UplInfoObject>,        // Upls that are depreciated or opened
  pub total_net: u32,                         // Total cart net value in HUF
  pub total_vat: u32,                         // Total VAT
  pub total_gross: u32,                       // Total cart gross value in HUF
  pub document_kind: DocumentKind,            // Receipt or Invoice
  pub payment_kind: PaymentKind,              // cash, transfer, card
  pub payments: Vec<Payment>,                 // Payment vector
  pub burned_points: Vec<LoyaltyTransaction>, // Burned payment points
  pub payable: i32,                           // Payable amount
  pub owner_uid: u32,                         // Shop assistant UID
  pub store_id: Option<u32>,                  // Now its stock ID
  pub date_completion: DateTime<Utc>,         // Invoice Completion date
  pub payment_duedate: DateTime<Utc>,         // Invoice Payment duedate
  pub created_by: u32,                        // UID
  pub created_at: DateTime<Utc>,              // When cart created
}

impl Default for Cart {
  fn default() -> Self {
    Self {
      ancestor: None,
      id: Uuid::default(),
      customer: None,
      commitment: None,
      commitment_discount_value: 0,
      loyalty_card: None,
      shopping_list: Vec::new(),
      upls_sku: Vec::new(),
      upls_unique: Vec::new(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::default(),
      payment_kind: PaymentKind::default(),
      payments: Vec::default(),
      burned_points: Vec::default(),
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

impl Commitment {
  pub fn new(commitment_id: Uuid, commitment_percentage: u32) -> Self {
    Self {
      commitment_id,
      commitment_percentage,
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LoyaltyLevel {
  L1,
  L2,
}

impl Default for LoyaltyLevel {
  fn default() -> Self {
    Self::L1
  }
}

impl ToString for LoyaltyLevel {
  fn to_string(&self) -> String {
    match self {
      LoyaltyLevel::L1 => "L1".to_string(),
      LoyaltyLevel::L2 => "L2".to_string(),
    }
  }
}

impl LoyaltyLevel {
  pub fn from_str(str: &str) -> Result<Self, String> {
    match str {
      "l1" | "L1" => Ok(Self::L1),
      "l2" | "L2" => Ok(Self::L2),
      _ => Err("Ismeretlen kedvezmény kártya szint. L1 | L2".to_string()),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoyaltyCard {
  pub account_id: Uuid,    // Loyalty account ID
  pub card_id: String,     // Loyalty card ID
  pub level: LoyaltyLevel, // L1 | L2
}

impl Default for LoyaltyCard {
  fn default() -> Self {
    Self {
      account_id: Uuid::default(),
      card_id: String::default(),
      level: LoyaltyLevel::default(),
    }
  }
}

impl LoyaltyCard {
  pub fn new(account_id: Uuid, card_id: String, level: LoyaltyLevel) -> Self {
    Self {
      account_id,
      card_id,
      level,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ListItem {
  pub sku: u32,
  pub name: String,
  pub piece: u32,
  pub vat: VAT,
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
    vat: VAT,
    unit_price_net: u32,
    unit_price_gross: u32,
  ) -> Self {
    let total_net = unit_price_net * piece;
    let total_gross = total_net * vat;
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
    self.total_price_gross = self.total_price_net * self.vat;
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
      vat: VAT::default(),
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

impl CartMethods for Cart {
  fn new(owner_uid: u32, store_id: Option<u32>, created_by: u32) -> Self {
    Self {
      ancestor: None,
      id: Uuid::new_v4(),
      customer: None,
      commitment: None,
      commitment_discount_value: 0,
      loyalty_card: None,
      shopping_list: Vec::default(),
      upls_sku: Vec::default(),
      upls_unique: Vec::default(),
      total_net: 0,
      total_vat: 0,
      total_gross: 0,
      document_kind: DocumentKind::Receipt,
      payment_kind: PaymentKind::Cash,
      payments: Vec::default(),
      burned_points: Vec::default(),
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
    vat: VAT,
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
    // Set DocumentKind
    self.document_kind = document_kind;
    self
  }

  fn get_payment_total(&self) -> i32 {
    self.payments.iter().map(|p| p.amount).sum()
  }

  fn set_payment(&mut self, payment_kind: PaymentKind) -> &Self {
    // Set payment duedate
    match &payment_kind {
      // Set 30 days if transfer
      PaymentKind::Transfer => {
        self.payment_duedate = Utc::today().and_hms(0, 0, 0) + Duration::days(30)
      }
      // Today if cash or card or else
      _ => self.payment_duedate = Utc::today().and_hms(0, 0, 0),
    }

    // TODO! Maybe some validation before set new value?
    self.payment_kind = payment_kind;
    self.calculate_totals();
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
    // Check if document_kind::Invoice but no customer added
    // return error
    if let DocumentKind::Invoice = self.document_kind {
      if self.customer.is_none() {
        return Err(
          "A kosár nem zárható le! Számlaigény van beállítva, de a vásárló üres!".to_string(),
        );
      }
    }

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
    let _total_net = self.get_items_total_net()
      - ((self.get_commitment_discount_value() + self.get_burned_points_balance()) as f32 / 1.27)
        .round() as u32;
    let _total_gross = self.get_items_total_gross()
      - self.get_commitment_discount_value()
      - self.get_burned_points_balance();
    let _total_vat = _total_gross - _total_net;

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
    self.total_net = self.get_items_total_net()
      - ((self.get_commitment_discount_value() + self.get_burned_points_balance()) as f32 / 1.27)
        .round() as u32;

    // Set new total gross
    self.total_gross = self.get_items_total_gross()
      - self.get_commitment_discount_value()
      - self.get_burned_points_balance();

    // Set new total vat
    self.total_vat = self.total_gross - self.total_net;

    // Set payable
    self.payable = match self.payment_kind {
      PaymentKind::Cash => crate::rounding::round_huf(self.total_gross as i32),
      _ => self.total_gross as i32,
    };

    // Set commitment discount value
    self.commitment_discount_value = self.get_commitment_discount_value();
  }

  fn add_loyalty_card(
    &mut self,
    account_id: Uuid,
    card_id: String,
    loyalty_level: LoyaltyLevel,
  ) -> Result<&Self, String> {
    match &self.loyalty_card {
      Some(card) => Err(
        "A kosárhoz már van kedvezmény kártya rendelve! Törölje azt, mielőtt másikat adna hozzá!"
          .to_string(),
      ),
      None => {
        self.loyalty_card = Some(LoyaltyCard::new(account_id, card_id, loyalty_level));
        Ok(self)
      }
    }
  }

  fn remove_loyalty_card(&mut self) -> Result<&Self, String> {
    if self.loyalty_card.is_none() {
      return Err("A kosárhoz nincs kártya rendelve, így azt nem lehet törölni!".to_string());
    }
    match self.get_burned_points_balance() == 0 {
      true => {
        // Remove loyalty card
        self.loyalty_card = None;
        Ok(self)
      }
      false => Err("Kártyát akkor lehet törölni, ha a felhasznált pontok összege 0. Törölje a felhasznált pontokat!".to_string())
    }
  }

  fn burn_points(
    &mut self,
    loyalty_account_id: Uuid,
    transaction_id: Uuid,
    points_to_burn: i32,
  ) -> Result<&Self, String> {
    // Check if we have enough points to remove
    // if we want to remove
    if points_to_burn < 0 {
      // If we want to get out more points that we have in
      // return error
      if self.get_burned_points_balance() < points_to_burn.abs() as u32 {
        return Err(
          "Több pontot szeretnénk kivenni a kosárból, mint amennyit felhasználtunk hozzá!"
            .to_string(),
        );
      }
    }

    // Check if transaction is already in use
    if self
      .burned_points
      .iter()
      .find(|tr| tr.transaction_id == transaction_id)
      .is_some()
    {
      return Err("A kért tranzakció már a felhasznált pontok között szerepel a kosárban, így nem adható hozzá ismét!".to_string());
    }

    // Burn points
    self.burned_points.push(LoyaltyTransaction {
      loyalty_account_id,
      transaction_id,
      burned_points: points_to_burn,
      created_at: Utc::now(),
    });

    // Recalculate totals
    self.calculate_totals();

    // Return Ok self ref
    Ok(self)
  }

  fn add_commitment(
    &mut self,
    commitment_id: Uuid,
    commitment_percentage: u32,
  ) -> Result<&Self, String> {
    // Create commitment object
    let new_commitment = Commitment::new(commitment_id, commitment_percentage);
    // Set commitment
    self.commitment = Some(new_commitment);
    // Finally recalculate totals
    self.calculate_totals();
    // Return ok self ref
    Ok(self)
  }

  fn remove_commitment(&mut self) -> Result<&Self, String> {
    if self.commitment.is_none() {
      return Err(
        "A kosárhoz nincs hozzárendelt commitment, így azt nem lehet eltávolítani".to_string(),
      );
    }
    // Remove commitment
    self.commitment = None;
    // Recalculate totals
    self.calculate_totals();
    // Return ok self ref
    Ok(self)
  }

  fn get_burned_points_balance(&self) -> u32 {
    match self
      .burned_points
      .iter()
      .fold(0, |acc, t| acc + t.burned_points)
    {
      x if x > 0 => x as u32,
      _ => 0,
    }
  }

  fn get_commitment_discount_value(&self) -> u32 {
    match &self.commitment {
      Some(commitment) => {
        return (self.get_items_total_gross() as f32
          * (commitment.commitment_percentage as f32 / 100.0))
          .round() as u32
      }
      None => 0,
    }
  }

  fn get_items_total_net(&self) -> u32 {
    self
      .shopping_list
      .iter()
      .map(|i| i.total_price_net)
      .sum::<u32>()
      + self
        .upls_unique
        .iter()
        .map(|u| u.get_price_net())
        .sum::<u32>()
  }

  fn get_items_total_gross(&self) -> u32 {
    self
      .shopping_list
      .iter()
      .map(|i| i.total_price_gross)
      .sum::<u32>()
      + self
        .upls_unique
        .iter()
        .map(|u| u.get_price_gross())
        .sum::<u32>()
  }

  fn get_items_total_vat(&self) -> u32 {
    self.get_items_total_gross() - self.get_items_total_net()
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
  pub vat: VAT,
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
      vat: VAT::default(),
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
