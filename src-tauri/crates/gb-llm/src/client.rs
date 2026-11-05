//! LM Studio: its OpenAI-compatible chat, plus its own API for what that one
//! cannot say, which models exist and which of them are in memory.

use std::future::Future;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::error::LlmError;
use crate::sse::SseDecoder;
use crate::think::ThinkFilter;

const API_SUFFIX: &str = "/v1";
const NATIVE_MODELS_PATH: &str = "/api/v1/models";
const NATIVE_LOAD_PATH: &str = "/api/v1/models/load";
const LLM_TYPE: &str = "llm";
const JSON_TYPE: &str = "application/json";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const LIST_TIMEOUT: Duration = Duration::from_secs(5);
const LOAD_TIMEOUT: Duration = Duration::from_secs(300);
/// Long: LM Studio may be loading the model before its first word.
const SILENCE_TIMEOUT: Duration = Duration::from_secs(180);
/// Embedding models are listed too, and they cannot chat.
const EMBEDDING_MARKER: &str = "embed";
const TEMPERATURE: f64 = 0.7;
/// Reasoning would only leave the player staring at silence: the facts are
/// already measured, the model has nothing to work out.
const REASONING_EFFORT: &str = "none";
const MAX_REPLY_TOKENS: u32 = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Speaker {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChatMessage {
    pub role: Speaker,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    pub base_url: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct CompletionBody<'request> {
    model: &'request str,
    messages: &'request [ChatMessage],
    stream: bool,
    temperature: f64,
    max_tokens: u32,
    reasoning_effort: &'request str,
}

#[derive(Deserialize)]
struct ModelList {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

#[derive(Deserialize)]
struct NativeModelList {
    models: Vec<NativeModel>,
}

#[derive(Deserialize)]
struct NativeModel {
    key: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    loaded_instances: Vec<IgnoredAny>,
}

#[derive(Serialize)]
struct LoadBody<'request> {
    model: &'request str,
}

fn server_root(base_url: &str) -> &str {
    let base = base_url.trim().trim_end_matches('/');

    base.strip_suffix(API_SUFFIX).unwrap_or(base)
}

fn endpoint(base_url: &str, path: &str) -> String {
    format!("{}{API_SUFFIX}/{path}", server_root(base_url))
}

/// No system proxy: the model lives on this machine.
fn http_client() -> Result<Client, LlmError> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .no_proxy()
        .build()
        .map_err(unreachable)
}

fn unreachable(error: reqwest::Error) -> LlmError {
    LlmError::Unreachable(error.to_string())
}

fn malformed(error: serde_json::Error) -> LlmError {
    LlmError::Malformed(error.to_string())
}

async fn check_status(response: Response) -> Result<Response, LlmError> {
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();

    Err(LlmError::Rejected(format!("{status}: {body}")))
}

/// `None` when the server has no such API: it may be another OpenAI-compatible one.
async fn native_models(base_url: &str) -> Option<Vec<NativeModel>> {
    let response = http_client()
        .ok()?
        .get(format!("{}{NATIVE_MODELS_PATH}", server_root(base_url)))
        .timeout(LIST_TIMEOUT)
        .send()
        .await
        .ok()?;
    let text = response.error_for_status().ok()?.text().await.ok()?;
    let list: NativeModelList = serde_json::from_str(&text).ok()?;

    Some(list.models)
}

/// `None` when the server cannot say, or does not know the model.
pub async fn is_model_loaded(base_url: &str, model: &str) -> Option<bool> {
    native_models(base_url)
        .await?
        .into_iter()
        .find(|entry| entry.key == model)
        .map(|entry| !entry.loaded_instances.is_empty())
}

/// A last resort: a model loaded this way stays in memory until someone unloads
/// it, while one LM Studio loads for a request goes away when idle.
async fn load_model(base_url: &str, model: &str) -> Result<(), LlmError> {
    let body = serde_json::to_vec(&LoadBody { model }).map_err(malformed)?;
    let response = http_client()?
        .post(format!("{}{NATIVE_LOAD_PATH}", server_root(base_url)))
        .header(CONTENT_TYPE, JSON_TYPE)
        .timeout(LOAD_TIMEOUT)
        .body(body)
        .send()
        .await
        .map_err(unreachable)?;

    check_status(response).await.map(|_| ())
}

/// The models that can chat. LM Studio's own list has them all, while its
/// OpenAI-compatible one may hold only those already in memory.
pub async fn chat_models(base_url: &str) -> Result<Vec<String>, LlmError> {
    if let Some(models) = native_models(base_url).await {
        return Ok(models
            .into_iter()
            .filter(|entry| entry.kind == LLM_TYPE)
            .map(|entry| entry.key)
            .collect());
    }

    let response = http_client()?
        .get(endpoint(base_url, "models"))
        .timeout(LIST_TIMEOUT)
        .send()
        .await
        .map_err(unreachable)?;
    let text = check_status(response)
        .await?
        .text()
        .await
        .map_err(unreachable)?;
    let list: ModelList = serde_json::from_str(&text).map_err(malformed)?;

    Ok(list
        .data
        .into_iter()
        .map(|entry| entry.id)
        .filter(|id| !id.contains(EMBEDDING_MARKER))
        .collect())
}

async fn open_stream(request: &ChatRequest) -> Result<Response, LlmError> {
    let body = serde_json::to_vec(&CompletionBody {
        model: &request.model,
        messages: &request.messages,
        stream: true,
        temperature: TEMPERATURE,
        max_tokens: MAX_REPLY_TOKENS,
        reasoning_effort: REASONING_EFFORT,
    })
    .map_err(malformed)?;
    let response = http_client()?
        .post(endpoint(&request.base_url, "chat/completions"))
        .header(CONTENT_TYPE, JSON_TYPE)
        .body(body)
        .send()
        .await
        .map_err(unreachable)?;

    check_status(response).await
}

/// LM Studio loads a missing model by itself unless that is switched off, so a
/// refusal is the sign to load it here and ask once more.
async fn open_stream_loading(request: &ChatRequest) -> Result<Response, LlmError> {
    let refused = match open_stream(request).await {
        Err(error @ LlmError::Rejected(_)) => error,
        other => return other,
    };

    if is_model_loaded(&request.base_url, &request.model).await != Some(false) {
        return Err(refused);
    }

    load_model(&request.base_url, &request.model).await?;
    open_stream(request).await
}

async fn read_stream(response: Response, on_text: &mut impl FnMut(&str)) -> Result<(), LlmError> {
    let mut stream = response.bytes_stream();
    let mut decoder = SseDecoder::default();
    let mut filter = ThinkFilter::default();

    loop {
        let next = timeout(SILENCE_TIMEOUT, stream.next())
            .await
            .map_err(|_| LlmError::Timeout)?;
        let Some(bytes) = next else {
            break;
        };
        let decoded = decoder.push(&bytes.map_err(unreachable)?)?;
        let shown = filter.push(&decoded.text);

        if !shown.is_empty() {
            on_text(&shown);
        }

        if decoded.is_done {
            break;
        }
    }

    let rest = filter.finish();

    if !rest.is_empty() {
        on_text(&rest);
    }

    Ok(())
}

/// Streams a reply into `on_text`. When `cancel` resolves the connection is
/// dropped, which is what stops LM Studio from generating.
pub async fn stream_chat(
    request: &ChatRequest,
    cancel: impl Future<Output = ()>,
    mut on_text: impl FnMut(&str),
) -> Result<(), LlmError> {
    let work = async {
        let response = open_stream_loading(request).await?;

        read_stream(response, &mut on_text).await
    };

    tokio::select! {
        result = work => result,
        () = cancel => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::future::pending;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;

    use super::*;

    const READ_BUFFER_BYTES: usize = 8192;

    /// Serves one request with `reply` and hands back what it was sent.
    async fn serve_once(reply: String) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0; READ_BUFFER_BYTES];
            let read = socket.read(&mut buffer).await.unwrap();

            socket.write_all(reply.as_bytes()).await.unwrap();
            socket.shutdown().await.unwrap();

            String::from_utf8_lossy(&buffer[..read]).into_owned()
        });

        (address, server)
    }

    /// Answers every request by what `route` says about its first line.
    async fn serve(route: impl Fn(&str) -> String + Send + Sync + 'static) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());

        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                answer(socket, &route).await;
            }
        });

        address
    }

    async fn answer(mut socket: TcpStream, route: &impl Fn(&str) -> String) {
        let mut buffer = vec![0; READ_BUFFER_BYTES];
        let read = socket.read(&mut buffer).await.unwrap();
        let sent = String::from_utf8_lossy(&buffer[..read]);
        let reply = route(sent.lines().next().unwrap_or_default());

        socket.write_all(reply.as_bytes()).await.unwrap();
        socket.shutdown().await.unwrap();
    }

    fn http_reply(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn request(base_url: String) -> ChatRequest {
        ChatRequest {
            base_url,
            model: "test-model".to_owned(),
            messages: vec![ChatMessage {
                role: Speaker::User,
                content: "hello".to_owned(),
            }],
        }
    }

    #[test]
    fn the_address_may_be_written_several_ways() {
        let expected = "http://localhost:1234/v1/models";

        assert_eq!(endpoint("http://localhost:1234", "models"), expected);
        assert_eq!(endpoint(" http://localhost:1234/ ", "models"), expected);
        assert_eq!(endpoint("http://localhost:1234/v1/", "models"), expected);
    }

    #[tokio::test]
    async fn the_full_catalogue_is_offered_without_embedding_models() {
        let address = serve(|_| {
            let body = r#"{"models":[
                {"type":"llm","key":"qwen3","loaded_instances":[]},
                {"type":"embedding","key":"nomic","loaded_instances":[]}
            ]}"#;

            http_reply("200 OK", body)
        })
        .await;

        assert_eq!(chat_models(&address).await.unwrap(), vec!["qwen3"]);
    }

    #[tokio::test]
    async fn another_server_falls_back_to_the_openai_list() {
        let address = serve(|line| {
            if line.contains("/api/v1/") {
                return http_reply("404 Not Found", "");
            }

            http_reply(
                "200 OK",
                r#"{"data":[{"id":"gpt"},{"id":"text-embedding-3"}]}"#,
            )
        })
        .await;

        assert_eq!(chat_models(&address).await.unwrap(), vec!["gpt"]);
        assert_eq!(is_model_loaded(&address, "gpt").await, None);
    }

    #[tokio::test]
    async fn a_refused_chat_loads_the_model_and_asks_again() {
        let is_loaded = Arc::new(AtomicBool::new(false));
        let state = Arc::clone(&is_loaded);
        let address = serve(move |line| {
            if line.contains("/models/load") {
                state.store(true, Ordering::SeqCst);

                return http_reply("200 OK", "{}");
            }

            if line.contains("/api/v1/models") {
                let instances = if state.load(Ordering::SeqCst) { "[{}]" } else { "[]" };
                let body = format!(
                    r#"{{"models":[{{"type":"llm","key":"test-model","loaded_instances":{instances}}}]}}"#
                );

                return http_reply("200 OK", &body);
            }

            if state.load(Ordering::SeqCst) {
                http_reply(
                    "200 OK",
                    "data: {\"choices\":[{\"delta\":{\"content\":\"Ready\"}}]}\n\ndata: [DONE]\n\n",
                )
            } else {
                http_reply("400 Bad Request", "no model loaded")
            }
        })
        .await;

        assert_eq!(is_model_loaded(&address, "test-model").await, Some(false));

        let mut text = String::new();

        stream_chat(&request(address), pending(), |piece| text.push_str(piece))
            .await
            .unwrap();

        assert_eq!(text, "Ready");
        assert!(is_loaded.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn a_reply_streams_without_its_thoughts() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"<think>hm</think>Кон\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ь\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let (address, server) = serve_once(http_reply("200 OK", body)).await;
        let mut text = String::new();

        stream_chat(&request(address), pending(), |piece| text.push_str(piece))
            .await
            .unwrap();

        assert_eq!(text, "Конь");

        let sent = server.await.unwrap();

        assert!(sent.starts_with("POST /v1/chat/completions"));
        assert!(sent.contains(r#""stream":true"#));
        assert!(sent.contains(r#""reasoning_effort":"none""#));
        assert!(sent.contains(r#""role":"user""#));
    }

    #[tokio::test]
    async fn a_refusal_carries_the_servers_words() {
        let reply = http_reply("400 Bad Request", "no model loaded");
        let (address, _) = serve_once(reply).await;
        let result = stream_chat(&request(address), pending(), |_| {}).await;

        assert!(matches!(
            result,
            Err(LlmError::Rejected(detail)) if detail.contains("no model loaded")
        ));
    }

    #[tokio::test]
    async fn cancelling_ends_the_call() {
        // Bound but never answered: without the cancel this would hang.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let (trigger, cancelled) = oneshot::channel::<()>();
        let cancel = async move {
            let _ = cancelled.await;
        };

        trigger.send(()).unwrap();

        assert!(stream_chat(&request(address), cancel, |_| {}).await.is_ok());
    }
}
