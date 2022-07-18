use std::cell::RefCell;
use std::rc::Rc;
use std::thread::{self, JoinHandle};

use crate::config::{SqlifeConfig, ConnectionConfig};
use crate::db::sgdb::{Connection};
use crate::db::{Message, MessageResponse, SGDBRelay};
use crate::ui::setup_style;
use eframe::egui;
use eframe::CreationContext;
use flume::{Receiver, Sender};
use log::info;

use crate::ui::views::{run, CurrentView, MessageID, NewConnectionWindow};

pub struct AppData {
    pub new_connection_win: NewConnectionWindow,

    handle_db: Option<JoinHandle<()>>,
    pub current_connection: Option<usize>,

    pub tx_sgdb: Option<Sender<Message<MessageID>>>,
    pub rx_sgdb: Option<Receiver<MessageResponse<MessageID>>>,
}

pub struct Sqlife {
    pub view: CurrentView,
    pub config: SqlifeConfig,
    pub data: AppData,
}

impl Sqlife {
    pub fn switch_connection(&mut self, index: usize) {
        info!("Switching connection..");

        if let Some(tx_sgdb) = &self.data.tx_sgdb {
            tx_sgdb.send(Message::Close).unwrap();
        }

        info!("Dropping DB threads..");

        if let Some(handle) = self.data.handle_db.take() {
            handle.join().unwrap();
        }

        info!("DB threads dropped");

        let (tx_ui, rx_ui) = flume::unbounded();
        let (tx_db, rx_db) = flume::unbounded();

        let con: Connection = self.config.connections[index].clone().into();

        self.data.handle_db = Some(thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .worker_threads(4)
                .build()
                .unwrap();

            runtime.block_on(async move {
                let sgdb = con.acquire_sgdb().await.unwrap();
                let mut db = SGDBRelay::new(sgdb, tx_db, rx_ui).await;
                db.run().await;
            });
        }));

        self.data.tx_sgdb = Some(tx_ui);
        self.data.rx_sgdb = Some(rx_db);

        self.data.current_connection = Some(index);

        self.switch_view(CurrentView::DBView(Default::default()));

        info!("Changing view done");
    }

    pub fn switch_view(&mut self, view: CurrentView) {
        self.view = view;

        self.view.init(&mut self.data, &mut self.config);
    }

    pub fn new(
        cc: &CreationContext<'_>,
        mut config: SqlifeConfig,
        connection_index: Option<usize>,
    ) -> Box<Self> {
        setup_style(cc);
        info!("Setup view");

        let mut data = AppData {
            handle_db: None,
            tx_sgdb: None,
            rx_sgdb: None,
            new_connection_win: NewConnectionWindow::default(),
            current_connection: None
        };

        let mut view = CurrentView::HelloView;
        view.init(&mut data, &mut config);

        let mut app = Box::new(Self { data, view, config});

        if let Some(con) = connection_index {
            app.switch_connection(con);
        } else if !app.config.connections.is_empty() {
            app.switch_connection(0);
        }

        app
    }
}

impl eframe::App for Sqlife {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        run(self, ctx);
    }
}
