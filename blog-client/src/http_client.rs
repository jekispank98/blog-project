pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        HttpClient
    }

    pub fn get_post(&self, id: &str) -> String {
        format!("[HTTP] Fetched post {} (stub)", id)
    }
}
