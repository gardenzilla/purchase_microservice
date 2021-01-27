// SKU to CART
// SKU, Derived Product, Depreciated

use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait PurchaseMethods
where
  Self: Sized,
{
  fn new() -> Self;
  fn open(&mut self) -> &Self;
  fn set_payment_method(&mut self) -> &Self;
  fn add_payment(&mut self) -> &Self;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Purchase {
  pub id: Uuid,            // Cart ID
  pub payment_amount: i32, // Total payable amount (rounded to 0|5 if Cash)
  pub payments: Vec<()>,   //
  pub balance: i32,        //
  pub restored: bool,
}

impl Default for Purchase {
  fn default() -> Self {
    Self {
      id: Uuid::default(),
      payment_amount: 0,
      payments: Vec::default(),
      balance: 0,
      restored: false,
    }
  }
}

impl VecPackMember for Purchase {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}
