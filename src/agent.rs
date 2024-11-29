use genai::{
    self,
    chat::{ChatMessage, ChatRequest, ChatResponse},
    Client, ModelIden,
};

pub struct Agent {
    client: Client,
    model: ModelIden,
}

impl Default for Agent {
    fn default() -> Self {
        let client = genai::Client::default();

        let model = client
            .resolve_model_iden("gemini-1.5-flash-latest")
            .expect("Failed to resolve model");

        Self { client, model }
    }
}

impl Agent {
    pub async fn send(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, genai::Error> {
        let request = ChatRequest {
            messages,
            system: Some("Responda em português Brasil".to_string()),
            ..Default::default()
        };

        let response = self
            .client
            .exec_chat(&self.model.model_name, request, None)
            .await?;

        Ok(response)
    }
}
