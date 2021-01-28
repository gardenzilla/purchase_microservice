use chrono::Utc;
use gzlib::proto::{
  self,
  purchase::{
    cart_object, upl_info_object, CartInfoObject, CartObject, Customer, Payment, PaymentKind,
    PurchaseInfoObject, PurchaseObject, UplInfoObject,
  },
};
use proto::purchase::purchase_object;
use upl_info_object::{UplKindOpenedSku, UplKindSku};

use crate::{
  cart::{self, CartMethods},
  purchase,
};

pub enum ServiceError {
  InternalError(String),
  NotFound(String),
  AlreadyExists(String),
  BadRequest(String),
}

impl ServiceError {
  pub fn internal_error(msg: &str) -> Self {
    ServiceError::InternalError(msg.to_string())
  }
  pub fn not_found(msg: &str) -> Self {
    ServiceError::NotFound(msg.to_string())
  }
  pub fn already_exist(msg: &str) -> Self {
    ServiceError::AlreadyExists(msg.to_string())
  }
  pub fn bad_request(msg: &str) -> Self {
    ServiceError::BadRequest(msg.to_string())
  }
}

impl std::fmt::Display for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ServiceError::InternalError(msg) => write!(f, "{}", msg),
      ServiceError::NotFound(msg) => write!(f, "{}", msg),
      ServiceError::AlreadyExists(msg) => write!(f, "{}", msg),
      ServiceError::BadRequest(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::fmt::Debug for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("")
      .field(&"ServiceError".to_string())
      .field(self)
      .finish()
  }
}

impl From<ServiceError> for ::tonic::Status {
  fn from(error: ServiceError) -> Self {
    match error {
      ServiceError::InternalError(msg) => ::tonic::Status::internal(msg),
      ServiceError::NotFound(msg) => ::tonic::Status::not_found(msg),
      ServiceError::AlreadyExists(msg) => ::tonic::Status::already_exists(msg),
      ServiceError::BadRequest(msg) => ::tonic::Status::invalid_argument(msg),
    }
  }
}

impl From<::packman::PackError> for ServiceError {
  fn from(error: ::packman::PackError) -> Self {
    match error {
      ::packman::PackError::ObjectNotFound => ServiceError::not_found(&error.to_string()),
      _ => ServiceError::internal_error(&error.to_string()),
    }
  }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl From<std::env::VarError> for ServiceError {
  fn from(error: std::env::VarError) -> Self {
    ServiceError::internal_error(&format!("ENV KEY NOT FOUND. {}", error))
  }
}

impl From<crate::cart::Cart> for CartInfoObject {
  fn from(f: crate::cart::Cart) -> Self {
    let mut names = f
      .shopping_list
      .iter()
      .map(|i| i.name.clone())
      .collect::<Vec<String>>();

    names.extend(
      f.upls_unique
        .iter()
        .map(|i| i.name.clone())
        .collect::<Vec<String>>(),
    );

    Self {
      cart_id: f.id.to_string(),
      customer_name: match f.customer {
        Some(c) => c.name.clone(),
        None => "".to_string(),
      },
      upl_count: f.upls_sku.len() as u32 + f.upls_unique.len() as u32,
      item_names: names,
      owner: f.owner_uid,
      created_by: f.created_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}

impl From<crate::cart::Cart> for CartObject {
  fn from(f: crate::cart::Cart) -> Self {
    Self {
      ancestor: match f.ancestor {
        Some(anc) => anc.to_string(),
        None => "".to_string(),
      },
      id: f.id.to_string(),
      customer: match &f.customer {
        Some(c) => Some(Customer {
          customer_id: c.id.clone(),
          name: c.name.clone(),
          zip: c.zip.clone(),
          location: c.location.clone(),
          street: c.street.clone(),
          tax_number: c.tax_number.clone(),
        }),
        None => None,
      },
      discount_percentage: f.discount_percentage.unwrap_or(0),
      shopping_list: f
        .shopping_list
        .iter()
        .map(|i| cart_object::Item {
          sku: i.sku,
          name: i.name.clone(),
          piece: i.piece,
          retail_price_net: i.unit_price_net,
          vat: i.vat.clone(),
          retail_price_gross: i.unit_price_gross,
          total_retail_price_net: i.total_price_net,
          total_retail_price_gross: i.total_price_gross,
        })
        .collect(),
      upls_sku: f
        .upls_sku
        .iter()
        .map(|uio| UplInfoObject {
          upl_id: uio.upl_id.clone(),
          upl_kind: Some(match uio.kind {
            cart::UplKind::Sku { sku, piece } => upl_info_object::UplKind::Sku(UplKindSku {
              sku: sku,
              piece: piece,
            }),
            cart::UplKind::DerivedProduct { product_id, amount } => {
              upl_info_object::UplKind::OpenedSku(UplKindOpenedSku {
                product_id: product_id,
                amount: amount,
              })
            }
          }),
          name: uio.name.clone(),
          retail_net_price: uio.retail_net_price,
          vat: uio.vat.clone(),
          retail_gross_price: uio.retail_gross_price,
          procurement_net_price: uio.procurement_net_price,
          best_before: match uio.best_before {
            Some(bb) => bb.to_rfc3339(),
            None => "".to_string(),
          },
          depreciated: uio.depreciated,
        })
        .collect(),
      upls_unique: f
        .upls_unique
        .iter()
        .map(|uio| UplInfoObject {
          upl_id: uio.upl_id.clone(),
          upl_kind: Some(match uio.kind {
            cart::UplKind::Sku { sku, piece } => upl_info_object::UplKind::Sku(UplKindSku {
              sku: sku,
              piece: piece,
            }),
            cart::UplKind::DerivedProduct { product_id, amount } => {
              upl_info_object::UplKind::OpenedSku(UplKindOpenedSku {
                product_id: product_id,
                amount: amount,
              })
            }
          }),
          name: uio.name.clone(),
          retail_net_price: uio.retail_net_price,
          vat: uio.vat.clone(),
          retail_gross_price: uio.retail_gross_price,
          procurement_net_price: uio.procurement_net_price,
          best_before: match uio.best_before {
            Some(bb) => bb.to_rfc3339(),
            None => "".to_string(),
          },
          depreciated: uio.depreciated,
        })
        .collect(),
      total_net: f.total_net,
      total_vat: f.total_vat,
      total_gross: f.total_gross,
      need_invoice: match f.document_kind {
        crate::cart::DocumentKind::Receipt => false,
        crate::cart::DocumentKind::Invoice => true,
      },
      payment_kind: match f.payment_kind {
        crate::cart::PaymentKind::Cash => PaymentKind::Cash,
        crate::cart::PaymentKind::Card => PaymentKind::Card,
        crate::cart::PaymentKind::Transfer => PaymentKind::Transfer,
      } as i32,
      payments: f
        .payments
        .iter()
        .map(|p| Payment {
          payment_id: p.payment_id.clone(),
          amount: p.amount,
        })
        .collect(),
      payment_balance: f.get_balance(),
      profit_net: f.get_profit_net(),
      owner_uid: f.owner_uid,
      store_id: f.store_id.unwrap_or(0), // 0 means no store
      date_completion: f.date_completion.to_rfc3339(),
      payment_duedate: f.payment_duedate.to_rfc3339(),
      created_by: f.created_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}

impl From<cart::Cart> for purchase::Purchase {
  fn from(f: cart::Cart) -> Self {
    let mut items: Vec<purchase::Item> = Vec::new();

    items.extend(
      f.shopping_list
        .iter()
        .map(|i| purchase::Item {
          kind: purchase::ItemKind::Sku, // Its just SKU
          product_id: 0, // 0 as its normal shopping list items, we wont need pid for invoice
          name: i.name.to_string(),
          piece: i.piece,
          retail_price_net: i.unit_price_net,
          vat: i.vat.to_string(),
          retail_price_gross: i.unit_price_gross,
          total_retail_price_net: i.total_price_net,
          total_retail_price_gross: i.total_price_gross,
        })
        .collect::<Vec<purchase::Item>>(),
    );

    items.extend(
      f.upls_unique
        .iter()
        .map(|i| purchase::Item {
          kind: match &i.kind {
            cart::UplKind::Sku { sku, piece } => match i.depreciated {
              true => purchase::ItemKind::DerivedProduct,
              false => purchase::ItemKind::Sku,
            },
            cart::UplKind::DerivedProduct {
              product_id: _,
              amount: _,
            } => purchase::ItemKind::DerivedProduct,
          },
          product_id: match &i.kind {
            _ => 0,
            cart::UplKind::DerivedProduct {
              product_id,
              amount: _,
            } => *product_id,
          },
          name: i.name.to_string(),
          piece: i.get_piece(),
          retail_price_net: i.retail_net_price,
          vat: i.vat.to_string(),
          retail_price_gross: i.retail_gross_price,
          total_retail_price_net: i.get_price_net(),
          total_retail_price_gross: i.get_price_gross(),
        })
        .collect::<Vec<purchase::Item>>(),
    );

    let mut upls: Vec<purchase::UplInfoObject> = Vec::new();

    upls.extend(
      f.upls_sku
        .iter()
        .map(|u| purchase::UplInfoObject {
          upl_id: u.upl_id.to_string(),
          kind: match u.kind {
            cart::UplKind::Sku { sku, piece } => purchase::UplKind::Sku { sku, piece },
            cart::UplKind::DerivedProduct { product_id, amount } => {
              purchase::UplKind::DerivedProduct { product_id, amount }
            }
          },
          name: u.name.to_string(),
          retail_net_price: u.retail_net_price,
          vat: u.vat.to_string(),
          retail_gross_price: u.retail_gross_price,
          procurement_net_price: u.procurement_net_price,
          best_before: match u.best_before {
            Some(bb) => Some(bb),
            None => None,
          },
          depreciated: u.depreciated,
        })
        .collect::<Vec<purchase::UplInfoObject>>(),
    );

    upls.extend(
      f.upls_unique
        .iter()
        .map(|u| purchase::UplInfoObject {
          upl_id: u.upl_id.to_string(),
          kind: match u.kind {
            cart::UplKind::Sku { sku, piece } => purchase::UplKind::Sku { sku, piece },
            cart::UplKind::DerivedProduct { product_id, amount } => {
              purchase::UplKind::DerivedProduct { product_id, amount }
            }
          },
          name: u.name.to_string(),
          retail_net_price: u.retail_net_price,
          vat: u.vat.to_string(),
          retail_gross_price: u.retail_gross_price,
          procurement_net_price: u.procurement_net_price,
          best_before: u.best_before,
          depreciated: u.depreciated,
        })
        .collect::<Vec<purchase::UplInfoObject>>(),
    );

    Self {
      id: f.id,
      customer: match &f.customer {
        Some(c) => Some(purchase::Customer {
          id: c.id,
          name: c.name.to_string(),
          zip: c.zip.to_string(),
          location: c.location.to_string(),
          street: c.street.to_string(),
          tax_number: c.tax_number.to_string(),
        }),
        None => None,
      },
      discount_percentage: f.discount_percentage,
      items: items,
      upl_info_objects: upls,
      total_net: f.total_net,
      total_vat: f.total_vat,
      total_gross: f.total_gross,
      document_kind: match f.document_kind {
        cart::DocumentKind::Receipt => purchase::DocumentKind::Receipt,
        cart::DocumentKind::Invoice => purchase::DocumentKind::Invoice,
      },
      payment_kind: match f.payment_kind {
        cart::PaymentKind::Cash => purchase::PaymentKind::Cash,
        cart::PaymentKind::Card => purchase::PaymentKind::Card,
        cart::PaymentKind::Transfer => purchase::PaymentKind::Transfer,
      },
      payments: f
        .payments
        .iter()
        .map(|p| purchase::Payment {
          payment_id: p.payment_id.to_string(),
          amount: p.amount,
        })
        .collect(),
      balance: f.get_balance(),
      profit_net: f.get_profit_net(),
      owner_uid: f.owner_uid,
      store_id: f.store_id,
      date_completion: Utc::today().and_hms(0, 0, 0),
      payment_duedate: Utc::today().and_hms(0, 0, 0), // TODO! refact to manage duedate for inv.
      restored: None,
      created_by: f.created_by,
      created_at: f.created_at,
    }
  }
}

impl From<purchase::Purchase> for PurchaseInfoObject {
  fn from(f: purchase::Purchase) -> Self {
    Self {
      purchase_id: f.id.to_string(),
      customer: match f.customer {
        Some(c) => Some(proto::purchase::Customer {
          customer_id: c.id,
          name: c.name,
          zip: c.zip,
          location: c.location,
          street: c.street,
          tax_number: c.tax_number,
        }),
        None => None,
      },
      upl_count: f.upl_info_objects.len() as u32,
      total_net_price: f.total_net,
      total_vat: f.total_vat,
      total_gross_price: f.total_gross,
      balance: f.balance,
      document_invoice: match f.document_kind {
        purchase::DocumentKind::Receipt => false,
        purchase::DocumentKind::Invoice => true,
      },
      date_completion: f.date_completion.to_rfc3339(),
      payment_duedate: f.payment_duedate.to_rfc3339(),
      payment_expired: (f.payment_duedate.date() > Utc::today()),
      profit_net: f.profit_net,
      restored: f.restored.is_some(),
      created_by: f.created_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}

impl From<purchase::Purchase> for PurchaseObject {
  fn from(f: purchase::Purchase) -> Self {
    Self {
      id: f.id.to_string(),
      customer: match f.customer {
        Some(c) => Some(proto::purchase::Customer {
          customer_id: c.id,
          name: c.name,
          zip: c.zip,
          location: c.location,
          street: c.street,
          tax_number: c.tax_number,
        }),
        None => None,
      },
      discount_percentage: f.discount_percentage.unwrap_or(0),
      items: f
        .items
        .iter()
        .map(|i| proto::purchase::purchase_object::Item {
          kind: match i.kind {
            purchase::ItemKind::Sku => purchase_object::ItemKind::Sku,
            purchase::ItemKind::SkuDepreciated => purchase_object::ItemKind::DepreciatedSku,
            purchase::ItemKind::DerivedProduct => purchase_object::ItemKind::DerivedProduct,
          } as i32,
          product_id: i.product_id,
          name: i.name.clone(),
          piece: i.piece,
          retail_price_net: i.retail_price_net,
          vat: i.vat.clone(),
          retail_price_gross: i.retail_price_gross,
          total_retail_price_net: i.total_retail_price_net,
          total_retail_price_gross: i.total_retail_price_gross,
          upl_ids: Vec::new(), // TODO remove this
        })
        .collect::<Vec<proto::purchase::purchase_object::Item>>(),
      upl_info_objects: Vec::new(),
      need_invoice: match f.document_kind {
        purchase::DocumentKind::Receipt => false,
        purchase::DocumentKind::Invoice => true,
      },
      total_net: f.total_net,
      total_vat: f.total_vat,
      total_gross: f.total_gross,
      payment_kind: match f.payment_kind {
        purchase::PaymentKind::Cash => proto::purchase::PaymentKind::Cash,
        purchase::PaymentKind::Card => proto::purchase::PaymentKind::Card,
        purchase::PaymentKind::Transfer => proto::purchase::PaymentKind::Transfer,
      } as i32,
      payments: f
        .payments
        .iter()
        .map(|p| proto::purchase::Payment {
          payment_id: p.payment_id.clone(),
          amount: p.amount,
        })
        .collect::<Vec<proto::purchase::Payment>>(),
      payment_balance: f.balance,
      profit_net: f.profit_net,
      owner_uid: f.owner_uid,
      store_id: f.store_id.unwrap_or(0),
      date_completion: f.date_completion.to_rfc3339(),
      payment_duedate: f.payment_duedate.to_rfc3339(),
      restored: f.restored.is_some(),
      created_by: f.created_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}
