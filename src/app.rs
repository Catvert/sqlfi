use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use crate::config::ConnectionConfig;
use crate::db::sgdb::{ConnectionSchema, SGDBBuilder};
use crate::db::{Message, SGDBRelay};
use crate::ui::{setup_style, views};
use eframe::egui;
use eframe::CreationContext;
use log::info;

use crate::ui::views::{run, CurrentView, DBData, MessageID, NewConnectionWindow};

pub struct AppData {
    selected_connection: Option<usize>,
    pub connections: Vec<ConnectionConfig>,
    pub new_connection_win: NewConnectionWindow,

    handle_ui: Option<JoinHandle<()>>,
    handle_db: Option<JoinHandle<()>>,

    pub schema: String,

    pub db_data: DBData,

    pub tx_sgdb: Option<Sender<Message<MessageID>>>,
}

pub struct Sqlife {
    pub view: CurrentView,
    pub data: AppData,
}

impl Sqlife {
    pub fn switch_connection<B: SGDBBuilder + Send + 'static>(
        &mut self,
        builder: B,
        view: CurrentView,
    ) {
        info!("Switching connection..");
        if let Some(tx_sgdb) = &self.data.tx_sgdb {
            tx_sgdb.send(Message::Close).unwrap();
        }

        info!("Dropping DB threads..");

        if let Some(handle) = self.data.handle_db.take() {
            handle.join().unwrap();
        }

        if let Some(handle) = self.data.handle_ui.take() {
            handle.join().unwrap();
        }

        info!("DB threads dropped");

        let (tx_ui, rx_ui) = mpsc::channel();
        let (tx_db, rx_db) = mpsc::channel();

        self.data.handle_db = Some(thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .worker_threads(4)
                .build()
                .unwrap();

            runtime.block_on(async move {
                let sgdb = builder.acquire_sgdb().await.unwrap();
                let mut db = SGDBRelay::new(sgdb, tx_db, rx_ui).await;
                db.run().await;
            });
        }));

        let db_data = self.data.db_data.clone();
        self.data.handle_ui = Some(thread::spawn(move || {
            views::process_db_response(rx_db, db_data);
        }));

        self.data.tx_sgdb = Some(tx_ui);

        self.view = view;

        self.view.init(&mut self.data);

        info!("Changing view done");
    }

    pub fn new(cc: &CreationContext<'_>, connection: ConnectionConfig) -> Box<Self> {
        setup_style(cc);
        info!("Setup view");

        let view = CurrentView::HelloView;

        let data = AppData {
            handle_db: None,
            handle_ui: None,
            tx_sgdb: None,
            db_data: DBData::default(),
            schema: connection.schema.clone(),
            connections: vec![connection.clone()],
            selected_connection: None,
            new_connection_win: NewConnectionWindow::default(),
        };

        let mut app = Box::new(Self { data, view });

        app.switch_connection::<ConnectionSchema>(
            connection.into(),
            CurrentView::DBView(views::db_view::DBViewData::default()),
        );

        app
    }

    pub fn selected_connection(&self) -> Option<usize> {
        self.data.selected_connection.clone()
    }
}

impl eframe::App for Sqlife {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        run(self, ctx);
    }
}
