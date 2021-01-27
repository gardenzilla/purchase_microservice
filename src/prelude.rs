use gzlib::proto::purchase::{
  cart_object, purchase_object, upl_info_object, CartInfoObject, CartObject, Customer, Payment,
  PaymentKind, UplInfoObject,
};
use upl_info_object::{UplKindOpenedSku, UplKindSku};

use crate::cart::{self, CartMethods};

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
    Self {
      cart_id: f.id.to_string(),
      customer_name: match f.customer {
        Some(c) => c.name.clone(),
        None => "".to_string(),
      },
      upl_count: f.upl_info_objects.len() as u32,
      item_names: f.items.iter().map(|i| i.name.clone()).collect(),
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
          address: c.address.clone(),
          tax_number: c.tax_number.clone(),
        }),
        None => None,
      },
      items: f
        .items
        .iter()
        .map(|i| cart_object::Item {
          kind: match &i.kind {
            crate::cart::ItemKind::Sku { sku: _ } => cart_object::ItemKind::Sku,
            crate::cart::ItemKind::SkuDepreciated { upl_id: _ } => {
              cart_object::ItemKind::DepreciatedSku
            }
            crate::cart::ItemKind::DerivedProduct {
              upl_id,
              amount,
              unit,
            } => cart_object::ItemKind::DerivedProduct,
          } as i32,
          name: i.name.clone(),
          piece: i.piece,
          retail_price_net: i.retail_price_net,
          vat: i.vat.clone(),
          retail_price_gross: i.retail_price_gross,
          total_retail_price_net: i.retail_price_net * i.piece,
          total_retail_price_gross: i.retail_price_gross * i.piece,
          upl_ids: i.upl_ids.clone(),
        })
        .collect(),
      upl_info_objects: f
        .upl_info_objects
        .iter()
        .map(|uio| UplInfoObject {
          upl_id: uio.upl_id.clone(),
          name: uio.name.clone(),
          unit: uio.unit.clone(),
          retail_net_price: uio.retail_net_price,
          vat: uio.vat.clone(),
          retail_gross_price: uio.retail_gross_price,
          procurement_net_price: uio.procurement_net_price,
          best_before: uio.best_before.to_rfc3339(),
          depreciated: uio.depreciated,
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
        })
        .collect(),
      need_invoice: match f.document_kind {
        crate::cart::DocumentKind::Receipt => false,
        crate::cart::DocumentKind::Invoice => true,
      },
      payment_kind: match f.payment_kind {
        crate::cart::PaymentKind::Cash => PaymentKind::Cash,
        crate::cart::PaymentKind::Card => PaymentKind::Card,
        crate::cart::PaymentKind::Transfer { payment_duedate: _ } => PaymentKind::Transfer,
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
      created_by: f.created_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}
