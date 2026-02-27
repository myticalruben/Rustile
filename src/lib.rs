use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, EventMask,
            change_window_attributes, configure_window, map_window,
        },
        *,
    },
};

use crate::core::{Stack, WindowId, Workspace};

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
            workspaces: vec![Workspace {name:"1".to_string(), stack: Stack { focused: 1, clients: vec![1] } }],
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
                    // Eliminar la ventana del stack
                    let ws = &mut self.workspaces[self.current_workspace];
                    ws.stack.clients.retain(|&id| id != e.window);

                    // Recalcular el espacio para las que quedan
                    self.apply_layour()?;
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
        let screen = &self.conn.setup().roots[self.screen_num];
        let ws = &self.workspaces[self.current_workspace];
        let n = ws.stack.clients.len();

        if n == 0 {
            return Ok(());
        }

        // Layout de Columnas Simple:
        let width_per_window = screen.width_in_pixels as u32 / n as u32;
        let height = screen.height_in_pixels as u32;

        for (i, &win) in ws.stack.clients.iter().enumerate() {
            let x = i as u32 * width_per_window;

            let values = ConfigureWindowAux::default()
                .x(x as i32)
                .y(0)
                .width(width_per_window)
                .height(height);

            self.conn.configure_window(win, &values)?;
        }

        self.conn.flush()?;
        Ok(())
    }

    fn handle_map_request(&mut self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Añadir al stack del workspace actual
        self.workspaces[self.current_workspace].stack.add(win);

        // 2. Escuchar si la ventana se destruye o se mueve
        let attrs = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::ENTER_WINDOW | EventMask::STRUCTURE_NOTIFY);
        self.conn.change_window_attributes(win, &attrs)?;

        // 3. Mapear (mostrar) la ventana
        self.conn.map_window(win)?;

        // 4. Recalcular posiciones de TODAS las ventanas
        self.apply_layour()?;

        println!("Ventana añadida al stack: {:?}", win);
        Ok(())
    }
}
