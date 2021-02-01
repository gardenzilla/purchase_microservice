use std::path::PathBuf;

use cart::*;
use packman::*;
use purchase::*;
use purchase_microservice::*;

fn main() {
  let carts_old: VecPack<migration::cart::CartOld> =
    VecPack::load_or_init(PathBuf::from("data/old/carts")).expect("Error while loading old carts");

  let mut carts_new: VecPack<Cart> =
    VecPack::load_or_init(PathBuf::from("data/new/carts")).expect("Error while loading old carts");

  carts_old.iter().for_each(|c| {
    let new_cart: Cart = c.unpack().clone().into();
    carts_new
      .insert(new_cart)
      .expect("Error inserting new cart");
  });

  let p_old: VecPack<migration::purchase::PurchaseOld> =
    VecPack::load_or_init(PathBuf::from("data/old/purchases"))
      .expect("Error while loading old purchases");

  let mut p_new: VecPack<Purchase> = VecPack::load_or_init(PathBuf::from("data/new/purchases"))
    .expect("Error while loading old purchases");

  p_old.iter().for_each(|c| {
    let new_p: Purchase = c.unpack().clone().into();
    p_new.insert(new_p).expect("Error inserting new purchase");
  });
}
