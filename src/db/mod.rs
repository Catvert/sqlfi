pub mod sgdb;

use self::sgdb::{SGDBFetchResult, SGDBTable, SGDB};
use anyhow::{anyhow, Result};
use flume::{Receiver, Sender};

#[derive(Debug)]
pub enum Message<ID> {
    FetchTables { schema: String },
    FetchAll(ID, String, Option<Vec<String>>),
    Close,
}

#[derive(Debug)]
pub enum MessageResponse<ID: Clone> {
    FetchAllResult(ID, Result<SGDBFetchResult>),
    TablesResult(Result<Vec<SGDBTable>>),
}

pub struct SGDBRelay<ID: Clone> {
    tx: Sender<MessageResponse<ID>>,
    rx: Receiver<Message<ID>>,

    sgdb: Box<dyn SGDB>,
}

impl<ID: Clone> SGDBRelay<ID> {
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
                Message::FetchAll(id, query, params) => {
                    let res = self
                        .sgdb
                        .fetch_all(&query, params)
                        .await
                        .map(|res| MessageResponse::FetchAllResult(id.clone(), Ok(res)))
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
                    break;
                }
            }
        }
    }
}
