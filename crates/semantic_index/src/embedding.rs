use std::{array::TryFromSliceError, sync::Arc};

use util::http::{AsyncBody, HttpClient, HttpClientWithUrl, Method, Request as HttpRequest};

use anyhow::{anyhow, Context as _, Result};
use futures::AsyncReadExt;
use serde::{Deserialize, Serialize};

/// Ollama's embedding via nomic-embed-text is of length 768
pub const EMBEDDING_SIZE_TINY: usize = 768;
/// Ollama's embedding via mxbai-embed-large is of length 1024
pub const EMBEDDING_SIZE_XSMALL: usize = 1024;
/// OpenAI's text small embeddings are of length 1536
pub const EMBEDDING_SIZE_SMALL: usize = 1536;
/// OpenAI's text large embeddings are of length 3072
pub const EMBEDDING_SIZE_LARGE: usize = 3072;

#[derive(Clone, Copy)]
pub enum EmbeddingModel {
    OllamaNomicEmbedText,
    OllamaMxbaiEmbedLarge,
    OpenaiTextEmbedding3Small,
    OpenaiTextEmbedding3Large,
}

#[derive(Debug, Clone)]
pub enum Embedding {
    OllamaNomicEmbedText([f32; EMBEDDING_SIZE_TINY]),
    OllamaMxbaiEmbedLarge([f32; EMBEDDING_SIZE_XSMALL]),
    OpenaiTextEmbedding3Small([f32; EMBEDDING_SIZE_SMALL]),
    OpenaiTextEmbedding3Large([f32; EMBEDDING_SIZE_LARGE]),
}

pub(crate) fn normalize_vector(embedding: Vec<f32>) -> Vec<f32> {
    // TODO: Either use ndarray directly like this:
    //         let array = ndarray::Array1::from(self.embedding.clone());
    //         let norm = array.dot(&array).sqrt();
    //         array / norm
    // OR: use simd operations directly to calculate the norm and normalize the embedding ourselves
    let len = embedding.len();
    let mut norm = 0f32;

    for i in 0..len {
        norm += embedding[i] * embedding[i];
    }

    norm = norm.sqrt();

    embedding.iter().map(|x| x / norm).collect::<Vec<f32>>()
}

pub fn normalize_embedding(
    embedding: Vec<f32>,
    embedding_type: EmbeddingModel,
) -> Result<Embedding> {
    let embedding = normalize_vector(embedding);

    match embedding_type {
        EmbeddingModel::OllamaNomicEmbedText if embedding.len() == EMBEDDING_SIZE_TINY => {
            Ok(Embedding::OllamaNomicEmbedText(
                embedding
                    .try_into()
                    .map_err(|_| anyhow!("Failed to convert to [f32; {}]", EMBEDDING_SIZE_TINY))?,
            ))
        }
        EmbeddingModel::OllamaMxbaiEmbedLarge if embedding.len() == EMBEDDING_SIZE_XSMALL => {
            Ok(Embedding::OllamaMxbaiEmbedLarge(
                embedding.try_into().map_err(|_| {
                    anyhow!("Failed to convert to [f32; {}]", EMBEDDING_SIZE_XSMALL)
                })?,
            ))
        }
        EmbeddingModel::OpenaiTextEmbedding3Small if embedding.len() == EMBEDDING_SIZE_SMALL => {
            Ok(Embedding::OpenaiTextEmbedding3Small(
                embedding
                    .try_into()
                    .map_err(|_| anyhow!("Failed to convert to [f32; {}]", EMBEDDING_SIZE_SMALL))?,
            ))
        }
        EmbeddingModel::OpenaiTextEmbedding3Large if embedding.len() == EMBEDDING_SIZE_LARGE => {
            Ok(Embedding::OpenaiTextEmbedding3Large(
                embedding
                    .try_into()
                    .map_err(|_| anyhow!("Failed to convert to [f32; {}]", EMBEDDING_SIZE_LARGE))?,
            ))
        }
        _ => Err(anyhow!("Invalid or mismatched embedding size")),
    }
}

/// Trait for embedding providers. Text in, vector out.
pub trait EmbeddingProvider {
    async fn get_embedding(&self, text: String) -> Result<Embedding>;
}

pub struct OllamaEmbeddingProvider {
    client: Arc<dyn HttpClient>,
    model: EmbeddingModel,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

impl OllamaEmbeddingProvider {
    pub fn new(client: Arc<dyn HttpClient>, model: EmbeddingModel) -> Self {
        Self { client, model }
    }
}

impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn get_embedding(&self, text: String) -> Result<Embedding> {
        let request = OllamaEmbeddingRequest {
            model: match self.model {
                EmbeddingModel::OllamaNomicEmbedText => "nomic-embed-text".to_string(),
                EmbeddingModel::OllamaMxbaiEmbedLarge => "mxbai-embed-large".to_string(),
                _ => return Err(anyhow!("Invalid model")),
            },
            prompt: text,
        };

        let request = serde_json::to_string(&request)?;
        let mut response = self
            .client
            .post_json("http://localhost:11434/api/embeddings", request.into())
            .await
            .context("failed to embed")?;

        let mut body = Vec::new();
        response.body_mut().read_to_end(&mut body).await.ok();

        let response: OllamaEmbeddingResponse =
            serde_json::from_slice(body.as_slice()).context("Unable to pull response")?;

        normalize_embedding(response.embedding, self.model)
    }
}

pub struct OpenaiEmbeddingProvider {
    client: Arc<dyn HttpClient>,
    model: EmbeddingModel,
    api_key: String,
}

#[derive(Serialize)]
struct OpenaiEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OpenaiEmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OpenaiEmbeddingResponse {
    object: String,
    data: Vec<OpenaiEmbeddingData>,
    model: String,
}

impl OpenaiEmbeddingProvider {
    pub fn new(client: Arc<dyn HttpClient>, model: EmbeddingModel, api_key: String) -> Self {
        Self {
            client,
            model,
            api_key,
        }
    }
}

impl EmbeddingProvider for OpenaiEmbeddingProvider {
    async fn get_embedding(&self, text: String) -> Result<Embedding> {
        let request = OpenaiEmbeddingRequest {
            model: match self.model {
                EmbeddingModel::OpenaiTextEmbedding3Small => "text-embedding-3-small".to_string(),
                EmbeddingModel::OpenaiTextEmbedding3Large => "text-embedding-3-large".to_string(),
                _ => return Err(anyhow!("Invalid model")),
            },
            prompt: text,
        };

        let api_url = "https://api.openai.com/v1/";

        let uri = format!("{api_url}/embeddings");

        let request = HttpRequest::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .body(AsyncBody::from(serde_json::to_string(&request)?))?;

        let mut response = self.client.send(request).await.context("Failed to embed")?;

        let mut body = Vec::new();
        response.body_mut().read_to_end(&mut body).await.ok();

        let response: OpenaiEmbeddingResponse =
            serde_json::from_slice(body.as_slice()).context("Unable to pull response")?;

        if let Some(first_embedding) = response.data.first() {
            normalize_embedding(first_embedding.embedding.clone(), self.model)
        } else {
            Err(anyhow!("No embedding data found in response"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gpui::BackgroundExecutor;

    #[gpui::test]
    async fn test_ollama_embedding_provider(executor: BackgroundExecutor) {
        executor.allow_parking();

        let client = Arc::new(HttpClientWithUrl::new("http://localhost:11434/"));
        let provider =
            OllamaEmbeddingProvider::new(client.clone(), EmbeddingModel::OllamaNomicEmbedText);
        let embedding = provider
            .get_embedding("Hello, world!".to_string())
            .await
            .unwrap();

        match embedding {
            Embedding::OllamaNomicEmbedText(e) => assert_eq!(e.len(), EMBEDDING_SIZE_TINY),
            _ => panic!("Invalid embedding size"),
        }
    }

    #[gpui::test]
    async fn test_ollama_embedding_not_exactly_a_benchmark(executor: BackgroundExecutor) {
        executor.allow_parking();

        let client = Arc::new(HttpClientWithUrl::new("http://localhost:11434/"));
        let provider =
            OllamaEmbeddingProvider::new(client.clone(), EmbeddingModel::OllamaNomicEmbedText);

        let t_nomic = std::time::Instant::now();
        for i in 0..100 {
            let embedding = provider
                .get_embedding(format!("Hello, world! {}", i))
                .await
                .unwrap();

            match embedding {
                Embedding::OllamaNomicEmbedText(e) => assert_eq!(e.len(), EMBEDDING_SIZE_TINY),
                _ => panic!("Invalid embedding size"),
            }
        }
        dbg!(t_nomic.elapsed());

        let client = Arc::new(HttpClientWithUrl::new("http://localhost:11434/"));
        let provider =
            OllamaEmbeddingProvider::new(client.clone(), EmbeddingModel::OllamaMxbaiEmbedLarge);

        let t_mxbai = std::time::Instant::now();
        for i in 0..100 {
            let embedding = provider
                .get_embedding(format!("Hello, world! {}", i))
                .await
                .unwrap();

            match embedding {
                Embedding::OllamaMxbaiEmbedLarge(e) => assert_eq!(e.len(), EMBEDDING_SIZE_XSMALL),
                _ => panic!("Invalid embedding size"),
            }
        }
        dbg!(t_mxbai.elapsed());
    }

    #[gpui::test]
    fn test_normalize_embedding() {
        // Create an vector of size EMBEDDING_SIZE_TINY with all values set to 1.0
        let embedding = vec![1.0, 1.0, 1.0];

        let normalized = normalize_vector(embedding);

        let value: f32 = 1.0 / 3.0_f32.sqrt();

        assert_eq!(normalized, vec![value; 3]);
    }
}