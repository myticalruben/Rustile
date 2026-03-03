use std::collections::HashMap;
use std::process::{Command, Stdio};

use x11rb::connection::Connection;
use x11rb::protocol::{Event, xproto::*};

use xkeysym::Keysym;

use crate::core::{Action, KeyBinding, Layout, Stack, WindowId, Workspace};

const COLOR_FOCUS: u32 = 0xbd93f9;
const COLOR_NORMAL: u32 = 0x44475a;
const BORDER_WIDTH: u32 = 2;

pub struct Rustile<C: Connection> {
    conn: C,
    screen_num: usize,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    key_map: HashMap<(u16, u8), Action>,
    atom_wm_protocols: Atom,
    atom_wm_delete_window: Atom,
    atom_wm_type: Atom,
    atom_wm_type_dialog: Atom,
    atom_wm_type_utility: Atom,
    atom_wm_type_toolbar: Atom,
    atom_wm_type_splash: Atom,
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

        let atom_wm_type = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let atom_wm_type_dialog = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DIALOG")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let atom_wm_type_utility = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_UTILITY")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let atom_wm_type_toolbar = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_TOOLBAR")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let atom_wm_type_splash = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_SPLASH")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

        let mut workspaces = Vec::new();

        for i in 1..=9 {
            workspaces.push(Workspace::new(i, &i.to_string()));
        }

        Self {
            conn,
            screen_num,
            workspaces,
            current_workspace: 0,
            key_map: HashMap::new(),
            atom_wm_protocols: wm_protocols,
            atom_wm_delete_window: wm_delete_window,
            atom_wm_type,
            atom_wm_type_dialog,
            atom_wm_type_utility,
            atom_wm_type_toolbar,
            atom_wm_type_splash,
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

    // pub fn set_border(&mut self, width: u32) -> u32 { }

    pub fn go_to_workspace(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index == self.current_workspace || index >= self.workspaces.len() {
            return Ok(());
        }

        //1. Ocultamos ventanas del workspace actual
        let old_ws = &self.workspaces[self.current_workspace];
        for &win in &old_ws.stack.clients {
            self.conn.unmap_window(win)?;
        }

        //2. Cambiar el indice
        self.current_workspace = index;

        //3. Mostramos las ventanas del nuevo workspace
        let new_ws = &self.workspaces[self.current_workspace];
        for &win in &new_ws.stack.clients {
            self.conn.map_window(win)?;
        }

        //4. Aplicamos el layour y damos el foco
        self.apply_layout()?;

        if let Some(&first) = new_ws.stack.clients.first() {
            self.set_focus(first)?;
        }

        println!("cambiamos al workspace {}", self.current_workspace);
        self.conn.flush();
        Ok(())
    }

    pub fn set_background_color(&self, color: u32) -> Result<(), Box<dyn std::error::Error>> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let root = screen.root;

        // Cambiamos el atributo 'back_pixel' de la ventana raíz
        let values = ChangeWindowAttributesAux::default().background_pixel(color);

        self.conn.change_window_attributes(root, &values)?;

        // Es necesario limpiar la ventana para que el color se aplique
        self.conn.clear_area(false, root, 0, 0, 0, 0)?;
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

    fn should_float(&self, win: WindowId) -> bool {
        let cookie = self
            .conn
            .get_property(false, win, self.atom_wm_type, AtomEnum::ATOM, 0, 1024);

        if let Ok(reply) = cookie.and_then(|c| Ok(c.reply().map_err(|e| e))) {
            let types: Vec<u32> = reply
                .unwrap()
                .value32()
                .map(|it| it.collect())
                .unwrap_or_default();

            for t in types {
                if t == self.atom_wm_type_dialog
                    || t == self.atom_wm_type_utility
                    || t == self.atom_wm_type_splash
                {
                    return true;
                }
            }
        }

        false
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

    fn resize_client(
        &self,
        win: WindowId,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //IMPORTANTE: Restamos el doble del borde para que el tamano total
        //(ventana + bordes) cincida con el espacio del layout.
        let border_width = BORDER_WIDTH;

        //Evitamos valores negativos o cero que puedan causar errores en X11
        let width = if w > 2 * border_width {
            w - 2 * border_width
        } else {
            1
        };

        let height = if h > 2 * border_width {
            h - 2 * border_width
        } else {
            1
        };

        let values = ConfigureWindowAux::default()
            .x(x as i32)
            .y(y as i32)
            .width(width)
            .height(height)
            .border_width(border_width);

        self.conn.configure_window(win, &values)?;
        self.conn.flush()?;

        Ok(())
    }

    fn apply_layout(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ws = &self.workspaces[self.current_workspace];
        let screen = &self.conn.setup().roots[self.screen_num];
        let n = ws.stack.clients.len();

        // 1. Si no hay ventanas, no hacemos nada
        if n == 0 {
            return Ok(());
        }

        // 2. CASO ESPECIAL: Una sola ventana
        if n == 1 {
            let win = ws.stack.clients[0];
            // Forzamos 0,0 y el ancho/alto TOTAL de la pantalla
            self.resize_client(
                win,
                0,
                0,
                screen.width_in_pixels as u32,
                screen.height_in_pixels as u32,
            )?;
        }
        // 3. CASO: Varias ventanas (Master/Stack)
        else {
            let master_width = (screen.width_in_pixels as f32 * ws.layout.ratio) as u32;
            let stack_width = screen.width_in_pixels as u32 - master_width;

            // La primera ventana SIEMPRE empieza en x=0
            self.resize_client(
                ws.stack.clients[0],
                0,
                0,
                master_width,
                screen.height_in_pixels as u32,
            )?;

            let stack_count = n - 1;
            let stack_height = screen.height_in_pixels as u32 / stack_count as u32;

            for (i, &win) in ws.stack.clients.iter().skip(1).enumerate() {
                let y = i as u32 * stack_height;
                // Las del stack empiezan donde termina el master (x = master_width)
                self.resize_client(win, master_width, y, stack_width, stack_height)?;
            }
        }

        self.conn.flush()?;
        Ok(())
    }

    fn set_focus(&mut self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        let ws = &mut self.workspaces[self.current_workspace];

        // Guardamos el antiguo para quitarle el brillo
        let old_focus = ws.stack.focused;
        ws.stack.focused = win;

        //Actualizamos ambos visualmente
        if old_focus != 0 {
            self.apply_border(old_focus, false)?;
        }
        if win != 0 {
            self.apply_border(win, true)?;
            self.conn
                .set_input_focus(InputFocus::POINTER_ROOT, win, x11rb::CURRENT_TIME)?;
        }

        self.conn.flush()?;
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
                Action::Swap(dir) => {
                    if let Err(e) = self.handle_swap(*dir) {
                        eprintln!(
                            "Error cuando se quiere cambiar de posicion la ventana {:?}",
                            e
                        );
                    }
                }
                Action::KillClient => {
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
                    if let Err(e) = self.handle_move_focus(*dir) {
                        eprintln!("Error al mover el foco: {}", e)
                    }
                }
                Action::GoToWorkspace(idx) => {
                    self.go_to_workspace(*idx);
                }
                Action::ChangeRatio(delta) => {
                    {
                        let ws = &mut self.workspaces[self.current_workspace];
                        ws.layout.change_ratio(*delta);
                    }

                    self.apply_layout()?;
                }
            }
        }

        Ok(())
    }

    fn handle_map_request(&mut self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        /* 1. Añadir al stack del workspace actual
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
        Ok(())*/

        if self.should_float(win) {
            // --Logica de ventana flontante--
            // La centramos en la pantalla y no la agremamos al stack del Layout
            let screen = &self.conn.setup().roots[self.screen_num];
            let width = 600;
            let height = 400;
            let x = (screen.width_in_pixels as u32 - width) / 2;
            let y = (screen.height_in_pixels as u32 - height) / 2;

            let values = ConfigureWindowAux::default()
                .x(x as i32)
                .y(y as i32)
                .width(width)
                .height(height)
                .border_width(2);

            self.conn.configure_window(win, &values)?;
        } else {
            // --- Logica de Tiling (lo que ya tenias) ---
            //1. Añadir al stack del workspace actual
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
        }

        self.conn.map_window(win)?;
        self.set_focus(win)?;
        self.conn.flush()?;

        Ok(())
    }

    fn handle_move_focus(&mut self, direction: i32) -> Result<(), Box<dyn std::error::Error>> {
        let focused_color = 0xbd93f9;
        let normal_color = 0x44475a;

        // 1. Obtene el workspace actual
        let ws = &mut self.workspaces[self.current_workspace];
        if ws.stack.clients.len() < 2 {
            return Ok(());
        }

        // 2. Guardar la ventana que va a perder el foco para limpiar su borde
        let old_focus = ws.stack.focused;

        // 3. Rotar el foco en el stack
        ws.stack.rotate_focus(direction);
        let new_focus = ws.stack.focused;

        // 4. Actualizar bordes en X11
        // Pintar la vieja de color normal
        self.conn.change_window_attributes(
            old_focus,
            &ChangeWindowAttributesAux::default().border_pixel(normal_color),
        )?;

        // Pintar la nueva de color resaltado
        self.conn.change_window_attributes(
            new_focus,
            &ChangeWindowAttributesAux::default().border_pixel(focused_color),
        )?;

        // 5. Mover el foco de entrada del teclado
        self.conn
            .set_input_focus(InputFocus::POINTER_ROOT, new_focus, x11rb::CURRENT_TIME)?;

        self.conn.flush()?;
        Ok(())
    }

    fn handle_swap(&mut self, direction: i32) -> Result<(), Box<dyn std::error::Error>> {
        {
            //Modificamos el stack del workspace actual
            let ws = &mut self.workspaces[self.current_workspace];
            ws.stack.swap(direction);
        }

        // Al cambiar el orden en el vector, refrescamos la pantalla
        self.apply_layout()?;

        println!("Ventanas intercambiadas (direccion: {})", direction);
        Ok(())
    }

    fn apply_border(
        &self,
        win: WindowId,
        is_focused: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let color = if is_focused {
            COLOR_FOCUS
        } else {
            COLOR_NORMAL
        };

        //1. Ajustar el grosor del borde
        let config_values = ConfigureWindowAux::default().border_width(BORDER_WIDTH);
        self.conn.configure_window(win, &config_values)?;

        //2. Ajustar el color del borde
        let attr_values = ChangeWindowAttributesAux::default().border_pixel(color);
        self.conn.change_window_attributes(win, &attr_values)?;

        self.conn.flush()?;
        Ok(())
    }
}
