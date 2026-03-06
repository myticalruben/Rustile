use std::collections::{HashMap, HashSet};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::{Event, xproto::*};

use x11rb::wrapper::ConnectionExt as _;
use xkeysym::Keysym;
use xkeysym::key::aogonek;

use crate::core::{Action, KeyBinding, RustileConfig, WindowId, Workspace};

pub struct Rustile<C: Connection> {
    conn: C,
    screen_num: usize,
    workspaces: Vec<Workspace>,
    current_workspace: usize,
    atom_wm_protocols: Atom,
    key_map: HashMap<(u16, u8), Action>,
    atom_wm_delete_window: Atom,
    atom_wm_type: Atom,
    atom_wm_type_dialog: Atom,
    atom_wm_type_utility: Atom,
    _atom_wm_type_toolbar: Atom,
    atom_wm_type_splash: Atom,
    pub config: RustileConfig,
    pub floating_windows: HashSet<Window>,
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

        let _atom_wm_type_toolbar = conn
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
            _atom_wm_type_toolbar,
            atom_wm_type_splash,
            config: RustileConfig::default(),
            floating_windows: HashSet::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.init()?;
        println!("Esperando ventanas");

        // Escaneamos X11 y adoptamos las ventanas huérfanas antes de hacer nada más.
        if let Err(e) = self.adopt_existing_window() {
            eprintln!("⚠️ Error al adoptar ventanas preexistentes: {}", e);
        }

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

    pub fn set_config(&mut self, config: RustileConfig) {
        self.config = config;
    }

    pub fn set_window_workspace_tag(
        &mut self,
        win: Window,
        ws: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //Pedimos el ID del post-it a X11
        let atom_desktop = self
            .conn
            .intern_atom(false, b"_NET_WM_DESKTOP")?
            .reply()?
            .atom;

        //Pegamos el valor (el indice del workspace) en la ventana
        self.conn.change_property32(
            PropMode::REPLACE,
            win,
            atom_desktop,
            AtomEnum::CARDINAL,
            &[ws as u32],
        )?;
        Ok(())
    }

    pub fn adopt_existing_window(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Entramos a adoptar las ventanas");
        let screen = &self.conn.setup().roots[self.screen_num];
        let tree = self.conn.query_tree(screen.root)?.reply()?;

        let atom_desktop = self
            .conn
            .intern_atom(false, b"_NET_WM_DESKTOP")?
            .reply()?
            .atom;

        for win in tree.children {
            let attrs = self.conn.get_window_attributes(win)?.reply()?;
            if attrs.override_redirect {
                continue;
            }

            //Intentamos leer la etiqueta _NET_WM_DESKTOP
            let prop = self
                .conn
                .get_property(false, win, atom_desktop, AtomEnum::ANY, 0, 1)?
                .reply()?;

            let mut target_ws = self.current_workspace;
            let mut is_window_mine = false;

            // Si tiene la etiqueta, extraemos el numero
            if let Some(value) = prop.value32() {
                if let Some(ws) = value.into_iter().next() {
                    println!("🔍 [ÉXITO] Ventana {} pertenece al Workspace {}", win, ws);
                    target_ws = ws as usize;
                    is_window_mine = true;
                }
            }

            // Adoptamos si tiene nuestra etiqueta (anque este oculta)
            // O si no tiene etiqueta pero esta visible (ventana huerfana nueva)
            if is_window_mine || attrs.map_state != x11rb::protocol::xproto::MapState::UNMAPPED {
                // Prevenir errores si el indice guardado es mayor a nuestros workspaces
                let ws = target_ws.min(self.workspaces.len() - 1);

                //Suscribir a eventos
                let event_mask =
                    EventMask::ENTER_WINDOW | EventMask::FOCUS_CHANGE | EventMask::PROPERTY_CHANGE;
                let aux = ChangeWindowAttributesAux::default().event_mask(event_mask);
                self.conn.change_window_attributes(win, &aux)?;

                // Agegamos a su workspace correspondiente
                self.workspaces[ws].stack.clients.push(win);

                if ws == self.current_workspace {
                    self.conn.map_window(win)?;
                } else {
                    self.conn.unmap_window(win)?;
                }
            }
        }

        self.apply_layout()?;
        self.conn.flush()?;
        println!("✅ Ventanas adoptadas con memoria perfecta.");
        Ok(())
    }

    fn move_to_workspace(&mut self, target: usize) -> Result<(), Box<dyn std::error::Error>> {
        let current = self.current_workspace;

        //Si es el mismo workspace o el indice no existe, no hacemos nada
        if target == current || target >= self.workspaces.len() {
            return Ok(());
        }

        //.Extraer la ventana enfocada del workspace actual

        let focused_win = {
            let current_ws = &mut self.workspaces[current];
            if let Some(pos) = current_ws
                .stack
                .clients
                .iter()
                .position(|&id| id == current_ws.stack.focused)
            {
                let id = current_ws.stack.clients.remove(pos);

                //Actualizamos el foco del workspace viejo (a la ventana que quedo)
                let _ = current_ws.stack.focused
                    == current_ws.stack.clients.first().copied().unwrap_or(0);
                Some(id)
            } else {
                None
            }
        };

        //2. Si habia una ventana enfocada, la mandamos al nuevo workspace
        if let Some(win) = focused_win {
            let target_ws = &mut self.workspaces[target];
            target_ws.stack.clients.push(win);
            target_ws.stack.focused = win; // La ventana movida mantiene el foco alli
            self.set_window_workspace_tag(win, target)?;

            //3. IMPORTANTE: Como la ventana "se fue" a otro escritorio, debemos ocultarla (unmap)
            self.conn.unmap_window(win)?;

            //4. Refrescamos el layout del escritorio actual para que las que se quedaron ocupen el
            //   espacio vacio
            self.apply_layout()?;
        }

        self.conn.flush()?;
        Ok(())
    }

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
        self.conn.flush()?;
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

    fn should_window_float(&self, win: Window) -> Result<bool, Box<dyn std::error::Error>> {
        //Comprobar si es "hija" de otra ventana (WM_TRASIENT_FOR)
        let trasient_reply = self
            .conn
            .get_property(
                false,
                win,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::ANY,
                0,
                1,
            )?
            .reply()?;

        if trasient_reply.value32().is_some() {
            return Ok(true); //Es un pop-up dialogo hijo
        }

        //Comprobamos el tipo de ventana en EWHM (_NET_WM_WINDOW_TYPE)
        let atom_type = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")?
            .reply()?
            .atom;
        let atom_dialog = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DIALOG")?
            .reply()?
            .atom;
        let atom_utility = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_UTILITY")?
            .reply()?
            .atom;
        let atom_splash = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_SPLASH")?
            .reply()?
            .atom;

        let atom_dropdown = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")?
            .reply()?
            .atom;
        let atom_popup = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_POPUP_MENU")?
            .reply()?
            .atom;
        let atom_tooltip = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_TOOLTIP")?
            .reply()?
            .atom;
        let atom_menu = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_MENU")?
            .reply()?
            .atom;

        let type_reply = self
            .conn
            .get_property(false, win, atom_type, AtomEnum::ANY, 0, 1024)?
            .reply()?;

        if let Some(mut values) = type_reply.value32() {
            if values.any(|v| {
                v == atom_dialog
                    || v == atom_utility
                    || v == atom_splash
                    || v == atom_dropdown
                    || v == atom_popup
                    || v == atom_tooltip
                    || v == atom_menu
            }) {
                return Ok(true); // Es un dialogo, utilidad o pantalla de carga
            }
        }

        //Inspeccion profunda del WM_WINDOW_ROLE
        let atom_role = self.conn.intern_atom(false, b"WM_WINDOW_ROLE")?.reply()?.atom;
        let role_reply = self.conn.get_property(false, win, atom_role, AtomEnum::ANY, 0, 1024)?.reply()?;
        if let Some(value) = role_reply.value8(){
            if let Ok(role_str) = std::str::from_utf8(&value.collect::<Vec<u8>>()){
                // Atrapamos el dialogo y otros pop-ups comunes
                if role_str.contains("GtkFileChooserDialog") || role_str.contains("pop-up") || role_str.contains("bubble"){
                    return Ok(true);
                }
            }
        }

        //Inspeccion profunda del WM_CLASS 
        let atom_class = self.conn.intern_atom(false, b"WM_CLASS")?.reply()?.atom;
        let class_reply = self.conn.get_property(false, win, atom_class, AtomEnum::ANY, 0, 1024)?.reply()?;
        if let Some(value) = class_reply.value8(){
            if let Ok(class_str) = std::str::from_utf8(&value.collect::<Vec<u8>>()){
                // Atrapamo explicitamente el portal de GTK que vimos en el xprop
                if class_str.contains("xdg-desktop-portal-gtk"){
                    return Ok(true);
                }
            }
        }
    
        //Si no cumple nada de lo anterior, va al tiling
        Ok(false)
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
        let border_width = self.config.border_width;

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

        //Separamos las ventanas en dos grupos
        let mut tiled_clients = Vec::new();
        for &win in &ws.stack.clients {
            if self.floating_windows.contains(&win) {
                // Las flotantes las subimos encima de las demas para que no queden ocultas
                let aux = ConfigureWindowAux::default().stack_mode(StackMode::ABOVE);
                self.conn.configure_window(win, &aux)?;
            } else {
                //Guardamos las que si van hacer Tiling
                tiled_clients.push(win);
            }
        }

        let n = tiled_clients.len();

        let sw = screen.width_in_pixels as u32;
        let sh = screen.height_in_pixels as u32;
        let g = self.config.gap_size;

        // 1. Si no hay ventanas, no hacemos nada
        if n == 0 {
            return Ok(());
        }

        // 2. CASO ESPECIAL: Una sola ventana
        if n == 1 {
            // Forzamos 0,0 y el ancho/alto TOTAL de la pantalla
            self.resize_client(tiled_clients[0], g, g, sw - 2 * g, sh - 2 * g)?;
        }
        // 3. CASO: Varias ventanas (Master/Stack)
        else {
            let master_width = (sw as f32 * ws.layout.ratio) as u32;
            let stack_width = sw - master_width;

            // La primera ventana SIEMPRE empieza en x=0
            self.resize_client(
                tiled_clients[0],
                g,
                g,
                master_width - (g + g / 2), //Deja espacio a la deracha para el stack
                sh - 2 * g,
            )?;

            let stack_count = n - 1;
            let stack_height = sh / stack_count as u32;

            for (i, &win) in tiled_clients.iter().skip(1).enumerate() {
                let y = i as u32 * stack_height;
                // Las del stack empiezan donde termina el master (x = master_width)
                self.resize_client(
                    win,
                    master_width + g / 2,
                    y + g,
                    stack_width - (g + g / 2),
                    stack_height - 2 * g,
                )?;
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
                    self.go_to_workspace(*idx)?;
                }
                Action::MoveToWorkspace(idx) => {
                    self.move_to_workspace(*idx)?;
                }
                Action::ChangeRatio(delta) => {
                    {
                        let ws = &mut self.workspaces[self.current_workspace];
                        ws.layout.change_ratio(*delta);
                    }

                    self.apply_layout()?;
                }
                Action::Restart => {
                    println!("🔄 Reiniciando Rustile...");

                    // Obtenemos la ruta exacta de este binario
                    if let Some(exe_path) = std::env::args().next() {
                        // La funcion .exec() de Unix reemplaza el proceso actual.
                        // Si tiene exito, el codigo debajo de esta linea NUNCA se ejecutara.
                        let err = Command::new(&exe_path).exec();

                        //Si llegamos aqui, es porque hubo un error
                        eprintln!(
                            "❌ Error fatal al reiniciar la ruta '{}': {}",
                            exe_path, err
                        );
                    } else {
                        eprintln!("❌ No se pudo determinal la ruta del archivo");
                    }
                }
                Action::ToggleFloat => {
                    //Obtenemos la ventana que tiene el foco actualmente
                    let focused = self.workspaces[self.current_workspace].stack.focused;

                    if focused != 0 {
                        //Si ya era flotante, la quitamos. Si no lo era, la agregamos.
                        if self.floating_windows.contains(&focused) {
                            self.floating_windows.remove(&focused);
                            println!("Ventana {} ha vuelto al Tiling", focused);
                        } else {
                            self.floating_windows.insert(focused);

                            //La centramos en la pantalla cuando la hacemos flotante
                            let screen = &self.conn.setup().roots[self.screen_num];
                            let w = 800;
                            let h = 600;
                            let x = (screen.width_in_pixels as u32 - w) / 2;
                            let y = (screen.height_in_pixels as u32 - h) / 2;

                            self.resize_client(focused, x, y, w, h)?;
                            println!("Ventana {} ahora esta Flotando", focused);
                        }

                        //Re-calculamos el layout para que las demoas llenen el hueco
                        self.apply_layout()?;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_map_request(&mut self, win: WindowId) -> Result<(), Box<dyn std::error::Error>> {
        //Escuchar si la ventana se destruye o se mueve
        let attrs = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::ENTER_WINDOW | EventMask::STRUCTURE_NOTIFY);
        self.conn.change_window_attributes(win, &attrs)?;

        // La ventana pide ser ignorada?
        let attr = self.conn.get_window_attributes(win)?.reply()?;
        if attr.override_redirect {
            // Ignoramos la ventana por completo y salimos del evento
            return Ok(());
        }

        //consultamos al dectector
        let should_float = self.should_window_float(win).unwrap_or(false);

        if should_float {
            println!("Ventana {} dectectada como flotante automaticamente", win);
            self.floating_windows.insert(win);

            //Obtenemos el size que la ventana Pidio tener
            if let Ok(geom) = self
                .conn
                .get_geometry(win)
                .and_then(|cookie| Ok(cookie.reply()))
            {
                let screen = &self.conn.setup().roots[self.screen_num];
                let g = geom.unwrap().clone();
                let gw = g.width as u32;
                let gh = g.height as u32;

                //La centramos en la pantalla usando su size original
                let x = (screen.width_in_pixels as u32).saturating_sub(gw) / 2;
                let y = (screen.height_in_pixels as u32).saturating_sub(gh) / 2;

                //La posicionamos pero respetando el ancho y alto que Pidio
                self.resize_client(win, x, y, gw, gh)?;
            }
        }

        //Añadir al stack del workspace actual
        self.workspaces[self.current_workspace]
            .stack
            .clients
            .push(win);
        self.set_window_workspace_tag(win, self.current_workspace)?;

        //Mapear (mostrar) la ventana
        self.conn.map_window(win)?;

        //Recalcular posiciones de TODAS las ventanas
        self.apply_layout()?;
        println!("Ventana añadida al stack: {:?}", win);

        self.conn.map_window(win)?;
        self.set_focus(win)?;
        self.conn.flush()?;

        Ok(())
    }

    fn handle_move_focus(&mut self, direction: i32) -> Result<(), Box<dyn std::error::Error>> {
        let focused_color = self.config.color_focus;
        let normal_color = self.config.color_normal;

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
            self.config.color_focus
        } else {
            self.config.color_normal
        };

        //1. Ajustar el grosor del borde
        let config_values = ConfigureWindowAux::default().border_width(self.config.border_width);
        self.conn.configure_window(win, &config_values)?;

        //2. Ajustar el color del borde
        let attr_values = ChangeWindowAttributesAux::default().border_pixel(color);
        self.conn.change_window_attributes(win, &attr_values)?;

        self.conn.flush()?;
        Ok(())
    }
}
