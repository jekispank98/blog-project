pub struct GrpcClient;

impl GrpcClient {
    pub fn new() -> Self {
        GrpcClient
    }

    pub fn get_post(&self, id: &str) -> String {
        format!("[gRPC] Fetched post {} (stub)", id)
    }
}
