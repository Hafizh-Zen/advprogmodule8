use tonic::Request;
use tokio_stream::StreamExt; // for stream.next()

// 1) Include generated code
pub mod services {
    tonic::include_proto!("services");
}

use services::{
    // Payment client
    payment_service_client::PaymentServiceClient,
    PaymentRequest,
    // Transaction client
    transaction_service_client::TransactionServiceClient,
    TransactionRequest,
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
    let mut stream = txn_cli
        .get_transaction_history(txn_req)
        .await?
        .into_inner();

    println!("🔄 Streaming transactions:");
    while let Some(item) = stream.next().await {
        let record = item?;
        println!("  • {:?}", record);
    }

    Ok(())
}
