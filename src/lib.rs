use pyo3::prelude::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[pyclass]
struct HeartbeatClient {
    server_host: String,
    server_port: u16,
    heartbeat_interval: u64,
    channel_id: String,
    timeout_message: String,
    reconnect_interval: u64,
    running: Arc<AtomicBool>,
    current_backoff: Arc<tokio::sync::Mutex<u64>>,
}

impl HeartbeatClient {
    async fn heartbeat_loop(&self) {
        let websocket_uri = format!(
            "ws://{}:{}",
            self.server_host, self.server_port
        );

        while self.running.load(Ordering::Relaxed) {
            match connect_async(&websocket_uri).await {
                Ok((ws_stream, _)) => {
                    log::info!(
                        "已连接到心跳服务器 {}:{}，订阅: {}, 心跳间隔: {} 秒",
                        self.server_host,
                        self.server_port,
                        self.channel_id,
                        self.heartbeat_interval
                    );

                    // 重置退避时间
                    *self.current_backoff.lock().await = 1;

                    let (mut write, mut _read) = ws_stream.split();

                    // 发送注册消息
                    let registration_data = json!({
                        "channel": self.channel_id,
                        "timeout_message": self.timeout_message,
                        "timestamp": get_timestamp(),
                        "type": "register",
                    });

                    if let Ok(msg) = serde_json::to_string(&registration_data) {
                        if let Err(e) = write.send(Message::Text(msg.into())).await {
                            let backoff = self.get_backoff_and_increase().await;
                            log::error!("发送注册消息失败: {}，将在 {} 秒后重试连接", e, backoff);
                            sleep(Duration::from_secs(backoff)).await;
                            continue;
                        }
                        log::info!("已发送注册/超时消息");
                    }

                    // 循环发送心跳
                    while self.running.load(Ordering::Relaxed) {
                        let heartbeat_data = json!({
                            "channel": self.channel_id,
                            "timestamp": get_timestamp(),
                            "type": "heartbeat",
                        });

                        match serde_json::to_string(&heartbeat_data) {
                            Ok(msg) => {
                                if let Err(e) = write.send(Message::Text(msg.into())).await {
                                    let backoff = self.get_backoff_and_increase().await;
                                    log::warn!("WebSocket 连接已关闭: {}，将在 {} 秒后重试连接", e, backoff);
                                    sleep(Duration::from_secs(backoff)).await;
                                    break;
                                }
                                log::info!(
                                    "已发送心跳 {}:{}，订阅: {}",
                                    self.server_host,
                                    self.server_port,
                                    self.channel_id
                                );
                            }
                            Err(e) => {
                                log::error!("序列化心跳数据失败: {}", e);
                                let backoff = self.get_backoff_and_increase().await;
                                log::warn!("将在 {} 秒后重试连接", backoff);
                                sleep(Duration::from_secs(backoff)).await;
                                break;
                            }
                        }

                        sleep(Duration::from_secs(self.heartbeat_interval)).await;
                    }

                    if let Err(e) = write.close().await {
                        log::warn!("关闭 websocket 时出错: {}", e);
                    } else {
                        log::info!("心跳线程中已断开与服务器的连接");
                    }
                }
                Err(e) => {
                    let backoff = self.get_backoff_and_increase().await;
                    log::error!("心跳线程连接/运行出错: {}，将在 {} 秒后重试", e, backoff);
                    if self.running.load(Ordering::Relaxed) {
                        sleep(Duration::from_secs(backoff)).await;
                    }
                }
            }
        }
    }

    async fn get_backoff_and_increase(&self) -> u64 {
        let mut backoff = self.current_backoff.lock().await;
        let current = *backoff;
        // 指数退避，但不超过配置的重试间隔
        *backoff = (*backoff * 2).min(self.reconnect_interval);
        current
    }
}

#[pymethods]
impl HeartbeatClient {
    #[new]
    #[pyo3(signature = (server_host="chan.local".to_string(), server_port=14999, heartbeat_interval=60, channel_id="gp_factory".to_string(), timeout_message="gp_factory可能已停止运行(客户端: {client_address}) ".to_string(), reconnect_interval=60))]
    fn new(
        server_host: String,
        server_port: u16,
        heartbeat_interval: u64,
        channel_id: String,
        timeout_message: String,
        reconnect_interval: u64,
    ) -> Self {
        Self {
            server_host,
            server_port,
            heartbeat_interval,
            channel_id,
            timeout_message,
            reconnect_interval,
            running: Arc::new(AtomicBool::new(false)),
            current_backoff: Arc::new(tokio::sync::Mutex::new(1)),
        }
    }

    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyAny>> {
        if self.running.load(Ordering::Relaxed) {
            log::warn!("心跳线程已经在运行");
            return pyo3_async_runtimes::tokio::future_into_py(py, async move {
                Ok(Python::attach(|py| py.None()))
            });
        }

        self.running.store(true, Ordering::Relaxed);

        let running = Arc::clone(&self.running);
        let current_backoff = Arc::clone(&self.current_backoff);
        let server_host = self.server_host.clone();
        let server_port = self.server_port;
        let heartbeat_interval = self.heartbeat_interval;
        let channel_id = self.channel_id.clone();
        let timeout_message = self.timeout_message.clone();
        let reconnect_interval = self.reconnect_interval;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = HeartbeatClient {
                server_host,
                server_port,
                heartbeat_interval,
                channel_id,
                timeout_message,
                reconnect_interval,
                running,
                current_backoff,
            };
            client.heartbeat_loop().await;
            Ok(Python::attach(|py| py.None()))
        })
    }

    fn stop(&self) -> PyResult<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

fn get_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// A Python module implemented in Rust.
#[pymodule]
fn heartbeat_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<HeartbeatClient>()?;
    Ok(())
}
