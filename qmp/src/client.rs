use std::{path::Path, sync::Arc};

use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufStream}, net::{UnixSocket, UnixStream}, sync::{RwLock, broadcast, mpsc, oneshot}};

use crate::{
    Error, Result,
    types::{InvokeCommand, CommandResponse, Response},
};

struct StreamLoop {
    buffer: RwLock<BufStream<UnixStream>>,

    drop: RwLock<mpsc::Receiver<()>>,
    queue: RwLock<mpsc::Receiver<(InvokeCommand, oneshot::Sender<CommandResponse>)>>,

    active_command: RwLock<Option<oneshot::Sender<CommandResponse>>>,
    close: RwLock<broadcast::Sender<()>>,
}

pub struct Client {
    error: Arc<RwLock<Option<Error>>>,

    drop: mpsc::Sender<()>,
    queue: mpsc::Sender<(InvokeCommand, oneshot::Sender<CommandResponse>)>,

    on_close: broadcast::Receiver<()>,
}

impl Client {
    pub async fn connect<P: AsRef<Path>>(path: P) -> Result<Self> {
        let socket = UnixSocket::new_stream()?;
        let stream = socket.connect(path).await?;
    
        let (drop_tx, drop_rx) = mpsc::channel(1);
        let (queue_tx, queue_rx) = mpsc::channel(100);
        let (close_tx, close_rx) = broadcast::channel(1);

        let qmp_loop = StreamLoop {
            buffer: RwLock::new(BufStream::new(stream)),
            drop: RwLock::new(drop_rx),
            queue: RwLock::new(queue_rx),
            active_command: RwLock::new(None),
            close: RwLock::new(close_tx),
        };

        let error = Arc::new(RwLock::new(None));

        let error_in = Arc::clone(&error);
        tokio::spawn(async move {
            if let Err(e) = qmp_loop.start().await {
                eprintln!("QMP client error: {}", e);
                error_in.write().await.replace(e);
            }
        });
        
        Ok(Client {
            error: error,
            drop: drop_tx,
            queue: queue_tx,
            on_close: close_rx,
        })
    }

    pub async fn invoke(&self, command: InvokeCommand) -> Result<CommandResponse> {
        if let Some(_) = self.error.read().await.as_ref() {
            return Err(Error::Protocol("Error occurred in QMP client".to_string()));
        }
        let (response_tx, response_rx) = oneshot::channel();
        self.queue.send((command, response_tx)).await.map_err(|_| Error::ChannelClosed)?;
        let response = response_rx.await.map_err(|_| Error::ChannelClosed)?;
        Ok(response)
    }

    pub async fn on_close(&mut self) -> Result<()> {
        self.on_close.recv().await.map_err(|_| Error::ChannelClosed)?;
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.drop.try_send(());
    }
}

enum PollResult {
    Drop,
    Response(Response),
    Queue((InvokeCommand, oneshot::Sender<CommandResponse>)),
    Skip,
}

impl StreamLoop {
    async fn read(&self) -> Result<Option<Response>> {
        let mut line = String::new();
        let mut buffer = self.buffer.write().await;
        match buffer.read_line(&mut line).await.err() {
            Some(e) => {
                if e.kind() == std::io::ErrorKind::ConnectionReset || e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(None)
                } else {
                    Err(Error::IO(e))
                }
            },
            None => {
                if line.is_empty() {
                    Ok(None)
                } else {
                    println!("qmp read: {}", line.trim_end());
                    Ok(Some(serde_json::from_str(&line)?))
                }
            }
        }
    }

    async fn write(&self, command: InvokeCommand) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        let command_str = serde_json::to_string(&command)? + "\n";
        buffer.write_all(command_str.as_bytes()).await?;
        buffer.flush().await?;
        Ok(())
    }

    async fn ensure_handshake(&self) -> Result<()> {
        let greeting = self.read().await?.ok_or(Error::HandshakeMissing)?;
        match greeting {
            Response::Greeting(_) => {
                self.write(InvokeCommand::empty("qmp_capabilities")).await?;
                self.read().await?;
                Ok(())
            },
            _ => Err(Error::Protocol("Expected greeting message".to_string())),
        }
    }

    async fn poll(&self) -> Result<PollResult> {
        let mut drop = self.drop.write().await;
        let mut queue = self.queue.write().await;
        tokio::select! {
            _ = drop.recv() => {
                Ok(PollResult::Drop)
            },
            response = self.read() => {
                match response? {
                    Some(resp) => Ok(PollResult::Response(resp)),
                    None => {
                        Ok(PollResult::Drop)
                    },
                }
            },
            command = queue.recv() => {
                if let Some(command) = command {
                    Ok(PollResult::Queue(command))
                } else {
                    Ok(PollResult::Skip)
                }
            }
        }
    }

    pub async fn start(self) -> Result<()> {
        self.ensure_handshake().await?;
        loop {
            match self.poll().await? {
                PollResult::Drop => {
                    break;
                },
                PollResult::Response(response) => {
                    match response {
                        Response::Greeting(_) => unreachable!(),
                        Response::CommandResponse(response) => {
                            let mut active_command = self.active_command.write().await;
                            if let Some(callback) = active_command.take() {
                                callback.send(response).map_err(|_| Error::ChannelClosed)?;
                            }
                        },
                        Response::Event(event) => {
                            println!("qmp: {:?}", event);
                        },
                    }
                },
                PollResult::Queue((command, callback)) => {
                    let mut active_command = self.active_command.write().await;
                    *active_command = Some(callback);

                    self.write(command).await?;
                },
                PollResult::Skip => {
                    continue;
                },
            }
        }
        self.close.write().await.send(()).ok();
        Ok(())
    }
}
