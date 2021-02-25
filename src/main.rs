use cart::CartMethods;
use chrono::{DateTime, Utc};
use gzlib::proto::purchase::{
  purchase_server::*, CartBulkRequest, CartByIdRequest, CartNewRequest,
};
use packman::*;
use prelude::*;
use proto::purchase::{
  CartAddCustomerReuqest, CartAddPaymentRequest, CartAddSkuRequest, CartAddUplRequest,
  CartCloseRequest, CartIds, CartInfoObject, CartObject, CartRemoveCustomerRequest,
  CartRemoveSkuRequest, CartRemoveUplRequest, CartSetOwnerRequest, CartSetPaymentRequest,
  CartSetSkuPieceRequest, CartSetStoreRequest, PurchaseBulkRequest, PurchaseByIdRequest,
  PurchaseIds, PurchaseInfoObject, PurchaseObject,
};
use purchase_microservice::*;
use std::error::Error;
use std::path::PathBuf;
use std::{env, str::FromStr};
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use gzlib::proto;

struct PurchaseService {
  carts: Mutex<VecPack<cart::Cart>>,
  purchases: Mutex<VecPack<purchase::Purchase>>,
}

// Helper to try convert string to UUID
fn string_to_uuid(id: String) -> ServiceResult<Uuid> {
  Uuid::from_str(&id).map_err(|_| ServiceError::BadRequest(format!("A kért ID hibás: {}", id)))
}

impl PurchaseService {
  pub fn init(carts: VecPack<cart::Cart>, purchases: VecPack<purchase::Purchase>) -> Self {
    Self {
      carts: Mutex::new(carts),
      purchases: Mutex::new(purchases),
    }
  }
  async fn cart_new(&self, r: CartNewRequest) -> ServiceResult<CartObject> {
    // Create new cart
    let new_cart = cart::Cart::new(
      r.owner_id,
      match r.store_id {
        0 => None,
        x => Some(x),
      },
      r.created_by,
    );
    // Insert it to the carts DB
    let _ = self.carts.lock().await.insert(new_cart.clone())?;
    // Return new cart
    Ok(new_cart.into())
  }

  async fn cart_get_all(&self) -> ServiceResult<Vec<String>> {
    // Collect all ID
    let res = self
      .carts
      .lock()
      .await
      .iter()
      .map(|c| c.unpack().id.to_string())
      .collect::<Vec<String>>();
    // Return the cart IDs
    Ok(res)
  }

  async fn cart_get_by_id(&self, r: CartByIdRequest) -> ServiceResult<CartObject> {
    // Try to find cart by id
    let res = self
      .carts
      .lock()
      .await
      .find_id(
        &Uuid::from_str(&r.cart_id)
          .map_err(|_| ServiceError::bad_request("A kért kosár ID hibás"))?,
      )?
      .unpack()
      .clone();
    // Return it as cart object
    Ok(res.into())
  }

  async fn cart_get_info_bulk(&self, r: CartBulkRequest) -> ServiceResult<Vec<CartInfoObject>> {
    // Transform the IDs from Vec<String> to Vec<Uuid>
    let mut ids: Vec<Uuid> = Vec::new();
    for id in r.cart_ids {
      ids.push(
        Uuid::from_str(&id).map_err(|_| ServiceError::BadRequest("A kért ID hibás".to_string()))?,
      );
    }
    // Try to find and transform the suitable carts
    let res = self
      .carts
      .lock()
      .await
      .iter()
      .filter(|c| ids.contains(&c.unpack().id))
      .map(|c| c.unpack().clone().into())
      .collect::<Vec<CartInfoObject>>();
    Ok(res)
  }

  async fn cart_add_customer(&self, r: CartAddCustomerReuqest) -> ServiceResult<CartObject> {
    // Try to find cart and add customer
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .add_customer(Some(cart::Customer {
        id: r.customer_id,
        name: r.customer_name,
        zip: r.customer_zip,
        location: r.customer_location,
        street: r.customer_street,
        tax_number: r.tax_number,
      }))
      .clone();
    Ok(res.into())
  }

  async fn cart_remove_customer(&self, r: CartRemoveCustomerRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .add_customer(None)
      .clone();
    Ok(res.into())
  }

  async fn cart_add_sku(&self, r: CartAddSkuRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .add_sku(
        r.sku_id,
        r.piece,
        r.name,
        r.vat,
        r.retail_price_net,
        r.retail_price_gross,
      )
      .clone();
    Ok(res.into())
  }

  async fn cart_remove_sku(&self, r: CartRemoveSkuRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .remove_sku(r.sku_id)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn cart_add_upl(&self, r: CartAddUplRequest) -> ServiceResult<CartObject> {
    let u = r
      .upl
      .ok_or(ServiceError::internal_error("Missing UPL object!"))?;

    let new_upl_info_object = cart::UplInfoObject {
      upl_id: u.upl_id,
      kind: match u
        .upl_kind
        .ok_or(ServiceError::internal_error("Missing KIND!"))?
      {
        proto::purchase::upl_info_object::UplKind::Sku(s) => cart::UplKind::Sku {
          sku: s.sku,
          piece: s.piece,
        },
        proto::purchase::upl_info_object::UplKind::OpenedSku(os) => cart::UplKind::DerivedProduct {
          product_id: os.product_id,
          amount: os.amount,
        },
      },
      name: u.name,
      retail_net_price: u.retail_net_price,
      vat: u.vat,
      retail_gross_price: u.retail_gross_price,
      procurement_net_price: u.procurement_net_price,
      best_before: match u.best_before.len() > 0 {
        true => Some(
          DateTime::parse_from_rfc3339(&u.best_before)
            .map_err(|_| ServiceError::internal_error("A megadott dátum hibás"))?
            .with_timezone(&Utc),
        ),
        false => None,
      },
      depreciated: u.depreciated,
    };
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .add_upl(new_upl_info_object)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn cart_remove_upl(&self, r: CartRemoveUplRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .remove_upl(r.upl_id)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn cart_set_payment(&self, r: CartSetPaymentRequest) -> ServiceResult<CartObject> {
    let p: proto::purchase::PaymentKind = proto::purchase::PaymentKind::from_i32(r.payment_kind)
      .ok_or(ServiceError::internal_error("PaymentKind decode error"))?;

    let payment = match p {
      proto::purchase::PaymentKind::Cash => cart::PaymentKind::Cash,
      proto::purchase::PaymentKind::Card => cart::PaymentKind::Card,
      proto::purchase::PaymentKind::Transfer => cart::PaymentKind::Transfer,
    };

    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .set_payment(payment)
      .clone();
    Ok(res.into())
  }

  async fn cart_add_payment(&self, r: CartAddPaymentRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .add_payment(cart::Payment {
        payment_id: r.payment_id,
        amount: r.amount,
      })
      .clone();
    Ok(res.into())
  }

  async fn cart_set_sku_piece(&self, r: CartSetSkuPieceRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .set_sku_piece(r.sku, r.piece)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn cart_set_owner(&self, r: CartSetOwnerRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .set_owner(r.owner_uid)
      .clone();
    Ok(res.into())
  }

  async fn cart_set_store(&self, r: CartSetStoreRequest) -> ServiceResult<CartObject> {
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id)?)?
      .as_mut()
      .unpack()
      .set_store_id(if r.store_id == 0 {
        None
      } else {
        Some(r.store_id)
      })
      .clone();
    Ok(res.into())
  }

  async fn cart_close(&self, r: CartCloseRequest) -> ServiceResult<CartObject> {
    // Try to close cart!
    let res = self
      .carts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.cart_id.clone())?)?
      .as_mut()
      .unpack()
      .close_cart()
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();

    let purchase: purchase::Purchase = res.clone().into();

    // Convert it to purchase and save it!
    self.purchases.lock().await.insert(purchase)?;

    // Remove cart finally
    self
      .carts
      .lock()
      .await
      .remove_pack(&string_to_uuid(r.cart_id)?)?;

    Ok(res.into())
  }

  async fn purchase_get_by_id(&self, r: PurchaseByIdRequest) -> ServiceResult<PurchaseObject> {
    let res = self
      .purchases
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.purchase_id)?)?
      .as_mut()
      .unpack()
      .clone();
    Ok(res.into())
  }

  async fn purchase_get_all(&self) -> ServiceResult<Vec<String>> {
    let res = self
      .purchases
      .lock()
      .await
      .iter()
      .map(|p| p.unpack().id.to_string())
      .collect::<Vec<String>>();
    Ok(res)
  }

  async fn purchase_get_info_bulk(
    &self,
    r: PurchaseBulkRequest,
  ) -> ServiceResult<Vec<PurchaseInfoObject>> {
    // Transform bulk IDS to Uuid vector
    let mut ids: Vec<Uuid> = Vec::new();
    for id in r.purchase_ids {
      ids.push(
        Uuid::from_str(&id).map_err(|_| ServiceError::BadRequest("A kért ID hibás".to_string()))?,
      );
    }

    let res = self
      .purchases
      .lock()
      .await
      .iter()
      .filter(|p| ids.contains(&p.unpack().id))
      .map(|p| p.unpack().clone().into())
      .collect::<Vec<PurchaseInfoObject>>();

    Ok(res.into())
  }
}

#[tonic::async_trait]
impl Purchase for PurchaseService {
  async fn cart_new(
    &self,
    request: Request<proto::purchase::CartNewRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_new(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_get_all(
    &self,
    _request: Request<()>,
  ) -> Result<Response<proto::purchase::CartIds>, Status> {
    let cart_ids = self.cart_get_all().await?;
    Ok(Response::new(CartIds { cart_ids }))
  }

  async fn cart_get_by_id(
    &self,
    request: Request<proto::purchase::CartByIdRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_get_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type CartGetInfoBulkStream = tokio::sync::mpsc::Receiver<Result<CartInfoObject, Status>>;

  async fn cart_get_info_bulk(
    &self,
    request: Request<proto::purchase::CartBulkRequest>,
  ) -> Result<Response<Self::CartGetInfoBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.cart_get_info_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn cart_add_customer(
    &self,
    request: Request<proto::purchase::CartAddCustomerReuqest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_add_customer(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_remove_customer(
    &self,
    request: Request<proto::purchase::CartRemoveCustomerRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_remove_customer(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_add_sku(
    &self,
    request: Request<proto::purchase::CartAddSkuRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_add_sku(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_set_sku_piece(
    &self,
    request: Request<proto::purchase::CartSetSkuPieceRequest>,
  ) -> Result<Response<CartObject>, Status> {
    let res = self.cart_set_sku_piece(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_remove_sku(
    &self,
    request: Request<proto::purchase::CartRemoveSkuRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_remove_sku(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_add_upl(
    &self,
    request: Request<proto::purchase::CartAddUplRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_add_upl(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_remove_upl(
    &self,
    request: Request<proto::purchase::CartRemoveUplRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_remove_upl(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_set_document(
    &self,
    request: Request<proto::purchase::CartSetDocumentRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_set_payment(
    &self,
    request: Request<proto::purchase::CartSetPaymentRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_set_payment(request.into_inner()).await?;
    Ok(Response::new(res.into()))
  }

  async fn cart_add_payment(
    &self,
    request: Request<proto::purchase::CartAddPaymentRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_add_payment(request.into_inner()).await?;
    Ok(Response::new(res.into()))
  }

  async fn cart_set_owner(
    &self,
    request: Request<proto::purchase::CartSetOwnerRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_set_owner(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_set_store(
    &self,
    request: Request<proto::purchase::CartSetStoreRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_set_store(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_close(
    &self,
    request: Request<proto::purchase::CartCloseRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    let res = self.cart_close(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn cart_remove(
    &self,
    request: Request<proto::purchase::CartRemoveRequest>,
  ) -> Result<Response<()>, Status> {
    todo!()
  }

  async fn purchase_get_by_id(
    &self,
    request: Request<proto::purchase::PurchaseByIdRequest>,
  ) -> Result<Response<proto::purchase::PurchaseObject>, Status> {
    let res = self.purchase_get_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn purchase_get_all(
    &self,
    request: Request<()>,
  ) -> Result<Response<proto::purchase::PurchaseIds>, Status> {
    let purchase_ids = self.purchase_get_all().await?;
    Ok(Response::new(PurchaseIds { purchase_ids }))
  }

  type PurchaseGetInfoBulkStream = tokio::sync::mpsc::Receiver<Result<PurchaseInfoObject, Status>>;

  async fn purchase_get_info_bulk(
    &self,
    request: Request<proto::purchase::PurchaseBulkRequest>,
  ) -> Result<Response<Self::PurchaseGetInfoBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.purchase_get_info_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn puchase_create_invoice(
    &self,
    request: Request<proto::purchase::PurchaseCreateInvoiceRequest>,
  ) -> Result<Response<proto::purchase::PurchaseObject>, Status> {
    todo!()
  }

  async fn purchase_add_payment(
    &self,
    request: Request<proto::purchase::PurchaseAddPaymentRequest>,
  ) -> Result<Response<proto::purchase::PurchaseObject>, Status> {
    todo!()
  }

  async fn purchase_restore(
    &self,
    request: Request<proto::purchase::PurchaseRestoreRequest>,
  ) -> Result<Response<proto::purchase::PurchaseObject>, Status> {
    todo!()
  }

  async fn purchase_get_stat_by_interval(
    &self,
    request: Request<proto::purchase::E>,
  ) -> Result<Response<proto::purchase::PurchaseStatResponse>, Status> {
    todo!()
  }

  async fn cart_loyalty_card_add(
    &self,
    request: Request<proto::purchase::E>,
  ) -> Result<Response<CartObject>, Status> {
    todo!()
  }

  async fn cart_loyalty_card_remove(
    &self,
    request: Request<proto::purchase::E>,
  ) -> Result<Response<CartObject>, Status> {
    todo!()
  }

  async fn cart_burn_points(
    &self,
    request: Request<proto::purchase::BurnPointsRequest>,
  ) -> Result<Response<CartObject>, Status> {
    todo!()
  }

  async fn cart_commitment_add(
    &self,
    request: Request<proto::purchase::AddCommitmentRequest>,
  ) -> Result<Response<CartObject>, Status> {
    todo!()
  }

  async fn cart_commitment_remove(
    &self,
    request: Request<proto::purchase::RemoveCommitmentRequest>,
  ) -> Result<Response<CartObject>, Status> {
    todo!()
  }

  async fn purchase_set_invoice_id(
    &self,
    request: Request<proto::purchase::PurchaseSetInvoiceIdRequest>,
  ) -> Result<Response<PurchaseObject>, Status> {
    todo!()
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // Init CARTS database
  let carts: VecPack<cart::Cart> =
    VecPack::load_or_init(PathBuf::from("data/carts")).expect("Error while loading carts db");

  // Init PURCHASES database
  let purchases: VecPack<purchase::Purchase> =
    VecPack::load_or_init(PathBuf::from("data/purchases"))
      .expect("Error while loading purchases db");

  let addr = env::var("SERVICE_ADDR_PURCHASE")
    .unwrap_or("[::1]:50072".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(PurchaseServer::new(PurchaseService::init(carts, purchases)))
      .serve_with_shutdown(addr, async {
        let _ = rx.await;
      })
      .await
      .unwrap()
  });

  tokio::signal::ctrl_c().await?;

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
