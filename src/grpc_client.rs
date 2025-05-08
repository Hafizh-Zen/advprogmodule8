use tonic::Request;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tokio::sync::mpsc;

// 1) Include generated code
pub mod services {
    tonic::include_proto!("services");
}

use services::{
    // Payment
    payment_service_client::PaymentServiceClient,
    PaymentRequest,
    // Transaction
    transaction_service_client::TransactionServiceClient,
    TransactionRequest,
    // Chat
    chat_service_client::ChatServiceClient,
    ChatMessage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Unary Payment ---
    let mut pay_cli = PaymentServiceClient::connect("http://[::1]:50051").await?;
    let pay_req = Request::new(PaymentRequest {
        order_id: "user_123".into(),
        amount: 100.0,
    });
    let pay_resp = pay_cli.process_payment(pay_req).await?;
    println!("📝 PAYMENT RESP = {:?}", pay_resp.into_inner());

    // --- Server-streaming Transaction ---
    let mut txn_cli = TransactionServiceClient::connect("http://[::1]:50051").await?;
    let txn_req = Request::new(TransactionRequest {
        user_id: "user_123".into(),
    });
    let mut stream = txn_cli.get_transaction_history(txn_req).await?.into_inner();
    println!("🔄 Streaming transactions:");
    while let Some(Ok(record)) = stream.next().await {
        println!("  • {:?}", record);
    }

    // --- Bi-directional Chat ---
    // 1) Create channel for outgoing messages
    let (tx, rx) = mpsc::channel::<ChatMessage>(32);
    // 2) Connect client
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    let mut chat_cli = ChatServiceClient::new(channel);
    // 3) Spawn task to read user input and send to server
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() { continue; }
            let msg = ChatMessage {
                user_id: "user_123".into(),
                message: line,
            };
            if tx.send(msg).await.is_err() {
                eprintln!("Failed to send to chat stream");
                break;
            }
        }
    });
    // 4) Start chat RPC
    let response_stream = chat_cli
        .chat(Request::new(ReceiverStream::new(rx)))
        .await?
        .into_inner();
    println!("💬 Chat started—type a message:");
    // 5) Print server replies
    tokio::pin!(response_stream);
    while let Some(Ok(reply)) = response_stream.next().await {
        println!("Server says: {:?}", reply);
    }

    Ok(())
}
