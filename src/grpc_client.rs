use tonic::Request;

// 1) Include the generated code
pub mod services {
    tonic::include_proto!("services");
}

// 2) Bring the client type and request message into scope
use services::payment_service_client::PaymentServiceClient;
use services::PaymentRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 3) Connect to the server
    let mut client = PaymentServiceClient::connect("http://[::1]:50051").await?;

    // 4) Build the request
    let req = Request::new(PaymentRequest {
        order_id: "user_123".into(),
        amount: 100.0,
    });

    // 5) Send it!
    let resp = client.process_payment(req).await?;
    println!("📝 RESPONSE = {:?}", resp.into_inner());

    Ok(())
}
