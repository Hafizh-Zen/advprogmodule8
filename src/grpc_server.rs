use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// 1) Include the generated proto code
pub mod services {
    tonic::include_proto!("services");
}

// 2) Bring server traits & message types into scope
use services::{
    // Payment
    payment_service_server::{PaymentService, PaymentServiceServer},
    PaymentRequest, PaymentResponse,
    // Transaction
    transaction_service_server::{TransactionService, TransactionServiceServer},
    TransactionRequest, TransactionResponse,
    // Chat
    chat_service_server::{ChatService, ChatServiceServer},
    ChatMessage,
};

/// 3) Implement PaymentService (Unary)
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

/// 4) Implement TransactionService (Server-streaming)
#[derive(Default)]
pub struct MyTransactionService;
#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        println!("→ Received txn history request: {:?}", request.get_ref());
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            for i in 1..=30 {
                let record = TransactionResponse {
                    transaction_id: format!("txn{}", i),
                    amount: (i as f64) * 10.0,
                };
                if tx.send(Ok(record)).await.is_err() { break; }
                if i % 10 == 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// 5) Implement ChatService (Bi-directional streaming)
#[derive(Default)]
pub struct MyChatService;
#[tonic::async_trait]
impl ChatService for MyChatService {
    type ChatStream = ReceiverStream<Result<ChatMessage, Status>>;

    async fn chat(
        &self,
        request: Request<tonic::Streaming<ChatMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(32);

        // Spawn a task to echo back each incoming message
        tokio::spawn(async move {
            while let Some(Ok(msg)) = inbound.message().await {
                println!("→ Received chat: {:?}", msg);
                let reply = ChatMessage {
                    user_id: msg.user_id.clone(),
                    message: format!(
                        "Terima kasih telah melakukan chat kepada CS virtual, Pesan anda akan dibalas pada jam kerja. pesan anda : {}",
                        msg.message
                    ),
                };
                if tx.send(Ok(reply)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// 6) Launch all services
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    println!("🚀 Server listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(MyPaymentService::default()))
        .add_service(TransactionServiceServer::new(MyTransactionService::default()))
        .add_service(ChatServiceServer::new(MyChatService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
