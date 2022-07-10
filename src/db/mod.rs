pub mod sgdb;

use anyhow::{anyhow, Context, Result};

use std::sync::{mpsc::{Receiver, Sender}, Arc};

use self::sgdb::{Connection, SGDBFetchResult, SGDBTable, SGDB};

#[derive(Debug)]
pub enum Message<ID> {
    FetchTables { schema: String },
    FetchAll(ID, String),
    Close,
}

pub enum MessageResponse<ID: Copy> {
    FetchAllResult(ID, Result<SGDBFetchResult>),
    TablesResult(Result<Vec<SGDBTable>>),
    Closed,
}

pub struct SGDBRelay<ID: Copy> {
    tx: Sender<MessageResponse<ID>>,
    rx: Receiver<Message<ID>>,

    sgdb: Box<dyn SGDB>,
}

impl<ID: Copy> SGDBRelay<ID> {
    pub async fn new(
        sgdb: Box<dyn SGDB>,
        tx: Sender<MessageResponse<ID>>,
        rx: Receiver<Message<ID>>,
    ) -> Self {
        Self { sgdb, tx, rx }
    }

    pub async fn run(&mut self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                Message::FetchAll(id, query) => {
                    let res = self
                        .sgdb
                        .fetch_all(&query)
                        .await
                        .map(|res| MessageResponse::FetchAllResult(id, Ok(res)))
                        .unwrap_or_else(|err| {
                            MessageResponse::FetchAllResult(id, Err(anyhow!("{}", err)))
                        });

                    self.tx.send(res).unwrap();
                }
                Message::FetchTables { schema } => {
                    let res = self
                        .sgdb
                        .list_tables(&schema)
                        .await
                        .map(|res| MessageResponse::TablesResult(Ok(res)))
                        .unwrap_or_else(|err| {
                            MessageResponse::TablesResult(Err(anyhow!("{}", err)))
                        });

                    self.tx.send(res).unwrap();
                }
                Message::Close => {
                    self.tx.send(MessageResponse::Closed).unwrap();
                    break;
                }
            }
        }
    }
}
