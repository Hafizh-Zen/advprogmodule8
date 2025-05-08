use tonic::{transport::Server, Request, Response, Status};

// 1) Include the generated code
pub mod services {
    tonic::include_proto!("services");
}

// 2) Bring the server traits and message types into scope
use services::{
    payment_service_server::{PaymentService, PaymentServiceServer},
    PaymentRequest, PaymentResponse,
};

/// 3) Define your service struct
#[derive(Default)]
pub struct MyPaymentService;

/// 4) Implement the generated trait
#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<PaymentResponse>, Status> {
        // Log the incoming request
        println!("→ Received payment request: {:?}", request.get_ref());

        // For demo purposes, always succeed
        let resp = PaymentResponse {
            success: true,
            confirmation: format!("Order {} processed!", request.into_inner().order_id),
        };

        Ok(Response::new(resp))
    }
}

/// 5) Spin up the server in main()
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Listen on [::1]:50051
    let addr = "[::1]:50051".parse()?;
    println!("🚀 gRPC server listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(MyPaymentService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
