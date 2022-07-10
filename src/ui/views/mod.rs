mod db_view;
mod hello_view;

pub use db_view::DBView;
pub use hello_view::HelloView;

use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
};

use eframe::egui::Ui;

use crate::db::{sgdb::SGDBBuilder, Message, MessageResponse, SGDBRelay};

pub struct ShareDB<T>(Arc<Mutex<T>>);

impl<T> ShareDB<T> {
    pub fn new(default: T) -> Self {
        Self(Arc::new(Mutex::new(default)))
    }

    pub fn duplicate(self) -> (Self, Self) {
        (self.share(), self)
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.0.lock().unwrap()
    }

    pub fn set(&self, v: T) {
        *self.0.lock().unwrap() = v;
    }

    pub fn share(&self) -> Self {
        Self(self.0.clone())
    }

}

impl<T: Default> Default for ShareDB<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait QueryShareDB {
    fn query<ID>(&mut self, tx: Sender<Message<ID>>);
}


pub enum QueryState<T> {
    Success(T),
    Waiting,
    Ready,
    Error(String),
}

impl<T> Default for QueryState<T> {
    fn default() -> Self {
        Self::Ready
    }
}

impl<T> QueryState<T> {
    pub fn query<ID>(&mut self, tx: &Sender<Message<ID>>, msg: Message<ID>) {
        *self = QueryState::Waiting;
        tx.send(msg).unwrap();
    }
}

pub fn spawn_sgdb_relay<B: SGDBBuilder + Send + 'static, ID: Copy + Send + 'static>(
    builder: B,
    tx: Sender<MessageResponse<ID>>,
    rx: Receiver<Message<ID>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .unwrap();

        runtime.block_on(async move {
            let sgdb = builder.acquire_sgdb().await.unwrap();
            let mut db = SGDBRelay::new(sgdb, tx, rx).await;
            db.run().await;
        });
    })
}

pub trait View {
    fn show(&mut self, ui: &mut Ui);
    fn show_appbar(&mut self, ui: &mut Ui);
}
