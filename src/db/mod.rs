pub mod sgdb;

use anyhow::{Result, anyhow};

use std::sync::mpsc::{Receiver, Sender};

use self::sgdb::{SGDBFetchResult, SGDB, SGDBTable};

#[derive(Debug)]
pub enum Message<ID> {
    Connect,
    FetchTables { schema: String },
    FetchAll(ID, String),
    Close,
}

pub enum MessageResponse<ID: Copy> {
    FetchAllResult(ID, Result<SGDBFetchResult>),
    TablesResult(Result<Vec<SGDBTable>>),
    Connected,
    Closed,
}

pub struct DBRelay<ID: Copy> {
    sgdb: Box<dyn SGDB>,
    tx: Sender<MessageResponse<ID>>,
    rx: Receiver<Message<ID>>,
}

impl<ID: Copy> DBRelay<ID> {
    pub async fn new(
        sgdb: Box<dyn SGDB>,
        tx: Sender<MessageResponse<ID>>,
        rx: Receiver<Message<ID>>,
    ) -> Self {
        Self { sgdb, tx, rx }
    }
    pub async fn run(&self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                Message::Connect => {
                    self.tx.send(MessageResponse::Connected).unwrap();
                }
                Message::FetchAll(id, query) => {
                    let res = self
                        .sgdb
                        .fetch_all(&query)
                        .await
                        .map(|res| MessageResponse::FetchAllResult(id, Ok(res)))
                        .unwrap_or_else(|err| MessageResponse::FetchAllResult(id, Err(anyhow!("{}", err))));

                    self.tx.send(res).unwrap();
                }
                Message::Close => {
                    self.tx.send(MessageResponse::Closed).unwrap();
                    break;
                }
                Message::FetchTables { schema } => {
                    let res = self
                        .sgdb
                        .tables(&schema)
                        .await
                        .map(|res| MessageResponse::TablesResult(Ok(res)))
                        .unwrap_or_else(|err| MessageResponse::TablesResult(Err(anyhow!("{}", err))));

                    self.tx.send(res).unwrap();
                }
            }
        }
    }
}
