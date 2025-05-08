use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// 1) Include the generated code
pub mod services {
    tonic::include_proto!("services");
}

// 2) Bring server traits and message types into scope
use services::{
    // Payment service
    payment_service_server::{PaymentService, PaymentServiceServer},
    PaymentRequest, PaymentResponse,
    // Transaction service
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionRequest, TransactionResponse,
};

/// 3) Payment service implementation
#[derive(Default)]
pub struct MyPaymentService;

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<PaymentResponse>, Status> {
        println!("→ Received payment request: {:?}", request.get_ref());
        let resp = PaymentResponse {
            success: true,
            confirmation: format!("Order {} processed!", request.into_inner().order_id),
        };
        Ok(Response::new(resp))
    }
}

/// 4) Transaction service implementation
#[derive(Default)]
pub struct MyTransactionService;

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    /// The streaming response type
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        println!(
            "→ Received transaction history request: {:?}",
            request.get_ref()
        );

        // Create a channel with buffer size 4
        let (tx, rx) = mpsc::channel(4);

        // Spawn a task to produce 30 dummy transactions
        tokio::spawn(async move {
            for i in 1..=30 {
                let record = TransactionResponse {
                    transaction_id: format!("txn{}", i),
                    amount: (i as f64) * 10.0,
                };
                // Send it; if the client disconnects, break
                if tx.send(Ok(record)).await.is_err() {
                    break;
                }
                // Every 10 records, wait 1s to simulate delay
                if i % 10 == 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });

        // Wrap the receiver in a Stream and return
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// 5) In main(), spin up both services
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    println!("🚀 Server listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(MyPaymentService::default()))
        .add_service(TransactionServiceServer::new(MyTransactionService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
