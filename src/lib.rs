use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ChangeWindowAttributesAux, ConfigureWindowAux, EventMask, change_window_attributes,
            configure_window, map_window,
        },
        *,
    },
};

use crate::core::{WindowId, Workspace};

mod core;

pub struct Rustile<C: Connection> {
    conn: C,
    screen_num: usize,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
}

impl<C: Connection> Rustile<C> {
    pub fn new(conn: C, screen_num: usize) -> Self {
        Self {
            conn,
            screen_num,
            workspaces: vec![Workspace {
                name: "1".to_string(),
                stack: None,
            }],
            current_workspace: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.init()?;
        println!("Esperando ventanas");

        loop {
            let event = self.conn.wait_for_event()?;

            match event {
                Event::MapRequest(e) => {
                    self.handle_map_request(e.window)?;
                }
                Event::UnmapNotify(e) => {
                    println!("Ventana cerrada: {:?}", e.window)
                }

                _ => {}
            }
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        //obtenemos las pantallas
        let screen = &self.conn.setup().roots[self.screen_num];

        // Queremos enterarnos de:
        // 1. SubstructureRedirect: Cuando una ventana quiere mostrarse.
        // 2. SubstructureNotify: Cuando una ventana cambia de estado o se destruye.
        let values = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY);

        change_window_attributes(&self.conn, screen.root, &values)?;

        self.conn.flush()?;
        Ok(())
    }

    fn apply_layour(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn handle_map_request(&self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        let values = ConfigureWindowAux::default()
            .width(800)
            .height(600)
            .x(100)
            .y(100);

        configure_window(&self.conn, win, &values)?;
        map_window(&self.conn, win)?;
        self.conn.flush()?;

        println!("Ventana adoptada: {:?}", win);
        Ok(())
    }
}
