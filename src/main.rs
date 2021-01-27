use chrono::{DateTime, Utc};
use gzlib::proto::purchase::purchase_server::*;
use packman::*;
use prelude::*;
use proto::purchase::{CartInfoObject, PurchaseInfoObject};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};

use gzlib::proto;

pub mod cart;
pub mod prelude;
pub mod purchase;

struct PurchaseService {
  carts: Mutex<VecPack<cart::Cart>>,
  purchases: Mutex<VecPack<purchase::Purchase>>,
}

impl PurchaseService {
  pub fn init(carts: VecPack<cart::Cart>, purchases: VecPack<purchase::Purchase>) -> Self {
    Self {
      carts: Mutex::new(carts),
      purchases: Mutex::new(purchases),
    }
  }
}

#[tonic::async_trait]
impl Purchase for PurchaseService {
  async fn cart_new(
    &self,
    request: Request<proto::purchase::CartNewRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_get_all(
    &self,
    request: Request<()>,
  ) -> Result<Response<proto::purchase::CartIds>, Status> {
    todo!()
  }

  async fn cart_get_by_id(
    &self,
    request: Request<proto::purchase::CartByIdRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  type CartGetInfoBulkStream = tokio::sync::mpsc::Receiver<Result<CartInfoObject, Status>>;

  async fn cart_get_info_bulk(
    &self,
    request: Request<proto::purchase::CartBulkRequest>,
  ) -> Result<Response<Self::CartGetInfoBulkStream>, Status> {
    todo!()
  }

  async fn cart_add_customer(
    &self,
    request: Request<proto::purchase::CartAddCustomerReuqest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_remove_customer(
    &self,
    request: Request<proto::purchase::CartRemoveCustomerRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_add_sku(
    &self,
    request: Request<proto::purchase::CartAddSkuRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_remove_sku(
    &self,
    request: Request<proto::purchase::CartRemoveSkuRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_add_upl(
    &self,
    request: Request<proto::purchase::CartAddUplRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_remove_upl(
    &self,
    request: Request<proto::purchase::CartRemoveUplRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
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
    todo!()
  }

  async fn cart_add_payment(
    &self,
    request: Request<proto::purchase::CartAddPaymentRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_set_owner(
    &self,
    request: Request<proto::purchase::CartSetOwnerRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_set_store(
    &self,
    request: Request<proto::purchase::CartSetStoreRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
  }

  async fn cart_close(
    &self,
    request: Request<proto::purchase::CartCloseRequest>,
  ) -> Result<Response<proto::purchase::CartObject>, Status> {
    todo!()
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
    todo!()
  }

  async fn purchase_get_all(
    &self,
    request: Request<()>,
  ) -> Result<Response<proto::purchase::PurchaseIds>, Status> {
    todo!()
  }

  type PurchaseGetInfoBulkStream = tokio::sync::mpsc::Receiver<Result<PurchaseInfoObject, Status>>;

  async fn purchase_get_info_bulk(
    &self,
    request: Request<proto::purchase::PurchaseBulkRequest>,
  ) -> Result<Response<Self::PurchaseGetInfoBulkStream>, Status> {
    todo!()
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
