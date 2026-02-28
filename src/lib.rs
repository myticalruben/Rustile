/*use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, EventMask, GrabMode, KeyPressEvent, change_window_attributes
        },
        *,
    },
};*/

use std::collections::HashMap;
use std::process::{Command, Stdio};

use x11rb::connection::Connection;
use x11rb::protocol::{Event, xproto::*};

use xkeysym::Keysym;

use crate::core::{Action, KeyBinding, Stack, WindowId, Workspace};

pub mod core;
pub mod mods;

pub struct Rustile<C: Connection> {
    conn: C,
    screen_num: usize,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    key_map: HashMap<(u16, u8), Action>,
    atom_wm_protocols: Atom,
    atom_wm_delete_window: Atom,
}

impl<C: Connection> Rustile<C> {
    pub fn new(conn: C, screen_num: usize) -> Self {
        let wm_protocols = conn
            .intern_atom(false, b"WM_PROTOCOLS")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let wm_delete_window = conn
            .intern_atom(false, b"WM_DELETE_WINDOW")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        Self {
            conn,
            screen_num,
            workspaces: vec![Workspace {
                name: "1".to_string(),
                stack: Stack {
                    focused: 1,
                    clients: vec![1],
                },
            }],
            current_workspace: 0,
            key_map: HashMap::new(),
            atom_wm_protocols: wm_protocols,
            atom_wm_delete_window: wm_delete_window,
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

                Event::KeyPress(e) => {
                    self.handle_key_press(e)?;
                }

                Event::UnmapNotify(e) => {
                    {
                        // Eliminar la ventana del stack
                        let ws = &mut self.workspaces[self.current_workspace];
                        ws.stack.clients.retain(|&id| id != e.window);

                        // Si era la que tenia el foco, pasar el foco a la siguiente
                        if ws.stack.focused == e.window {
                            ws.stack.focused = ws.stack.clients.last().copied().unwrap_or(0);
                        }
                    }

                    self.apply_layout()?;

                    let focused_win = self.workspaces[self.current_workspace].stack.focused;
                    if focused_win != 0 {
                        self.set_focus(focused_win)?;
                    }
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

    pub fn setup_keybindings(&mut self, bindings: Vec<KeyBinding>) {
        let mut map = HashMap::new();

        for b in bindings {
            // Usamos la funcion que creamos antes para obtener el codigo fisico
            let code = self.get_keycode_from_keysym(b.key);

            // Insertamos en nuestro buscador rapido
            map.insert((b.modifiers, code), b.action);

            // Tambien le decimos a X11 que "agarre" esta tecla
            let screen = &self.conn.setup().roots[self.screen_num];
            self.conn
                .grab_key(
                    false,
                    screen.root,
                    b.modifiers.into(),
                    code,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .ok();

            self.key_map = map.clone();
            self.conn.flush().ok();
        }
    }

    fn execute_spawn(&self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Separamos el comando de los argumentos (ej: "firefox --private"-> "firefox", ["--private"] )

        let mut parts = cmd.split_whitespace();
        let program = match parts.next() {
            Some(p) => p,
            None => return Ok(()),
        };

        let args: Vec<&str> = parts.collect();

        // Lanzamos el proceso
        Command::new(program)
            .args(args)
            .stdin(Stdio::null()) //No queremos que hereden la entrada/salida
            .stdout(Stdio::null()) // del window Manager
            .stderr(Stdio::null())
            .spawn()?;

        Ok(())
    }

    fn get_keycode_from_keysym(&self, sym: Keysym) -> u8 {
        let setup = self.conn.setup();
        let min = setup.min_keycode;
        let max = setup.max_keycode;

        // Obtenemos el mapa actual del servidor X
        let mapping = self
            .conn
            .get_keyboard_mapping(min, max - min + 1)
            .unwrap()
            .reply()
            .expect("No se pudo obtener el mapa de teclado");

        // Buscamos la posicion de Keysym para obtener el Keycode fisico

        for (idx, syms) in mapping
            .keysyms
            .chunks(mapping.keysyms_per_keycode as usize)
            .enumerate()
        {
            for &s in syms {
                if s == sym.into() {
                    return min + idx as u8;
                }
            }
        }

        panic!("El servidor X no tiene un Keycode asignado para el Keysym")
    }

    fn apply_layout(&self) -> Result<(), Box<dyn std::error::Error>> {
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
        self.apply_layout()?;

        println!("Ventana añadida al stack: {:?}", win);
        Ok(())
    }

    fn set_focus(&self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        if win == 0 {
            return Ok(());
        }

        // Reclamar el foco de entrada para nuestro WM
        self.conn
            .set_input_focus(InputFocus::POINTER_ROOT, win, x11rb::CURRENT_TIME);

        let values = ChangeWindowAttributesAux::default().border_pixel(0xbd93f9);
        self.conn.change_window_attributes(win, &values)?;
        self.conn.flush()?;
        Ok(())
    }

    fn handle_key_press(&mut self, e: KeyPressEvent) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Extraer los datos del evento
        // e.state son los modificadores (Mod4, Shift, etc.)
        // e.detail es el Keycode físico
        let state = e.state;
        let code = e.detail;

        // 2. Buscar la accion en nuestro HashMap
        if let Some(action) = self.key_map.get(&(state.into(), code)) {
            match action {
                Action::Spawn(cmd) => {
                    self.execute_spawn(cmd)?;
                }
                Action::KillClient => {
                    /* let ws = &mut self.workspaces[self.current_workspace];
                    if let Some(win) = Some(ws.stack.focused).filter(|&id| id != 0) {
                        println!("Intentando cerrar ventana: {:?}", win);

                        //3. intentamos matar el cliente de forma limpia
                        //Si usamos x11rb, esto cierra la conexion del cliente con el servidor
                        self.conn.kill_client(win)?;

                        //4. IMPORTANTE: eliminarla de nuestro Stack local inmediatamente
                        ws.stack.clients.retain(|&id| id != win);

                        //5. Resetear el foco a la ventana anterior si queda alguna
                        if let Some(&next_win) = ws.stack.clients.last() {
                            ws.stack.focused = next_win;
                            self.set_focus(next_win)?;
                        } else {
                            ws.stack.focused = 0;
                        }

                        //5.Recalcular el layour para llenar el vacio
                        self.apply_layout()?;
                        self.conn.flush();
                    }*/

                    let focused_win = {
                        let ws = &self.workspaces[self.current_workspace];
                        ws.stack.focused
                    };

                    if focused_win != 0 {
                        //Intentamos el cierre elegante
                        if let Err(e) = self.close_window(focused_win) {
                            eprintln!("Error al intentar cerrar la ventana: {}", e);
                        }
                    }
                }
                Action::MoveFocus(dir) => {
                    let ws = &mut self.workspaces[self.current_workspace];
                    /*  ws.stack.rotate_focus(*dir);
                    let new_focus = ws.stack.focused;
                    self.set_focus(new_focus)?;*/
                }
                Action::GoToWorkspace(idx) => {
                    //self.switch_workspace(idx);
                }
            }
        }

        Ok(())
    }

    fn close_window(&self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        // Construimos el evento de mensaje
        // Los datos debe ir en un array de 5 elementos de 32 bits (u32)
        let data = [self.atom_wm_delete_window, 0, 0, 0, 0];

        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: win,
            type_: self.atom_wm_protocols,
            data: ClientMessageData::from(data),
        };

        // Enviamos el evento directamente a la ventana del cliente
        self.conn
            .send_event(false, win, EventMask::NO_EVENT, event)?;
        self.conn.flush()?;

        println!("Envio WM_DELETE_WINDOW para la ventana: {:?}", win);
        Ok(())
    }
}
