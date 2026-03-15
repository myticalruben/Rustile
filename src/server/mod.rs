mod state;

use std::{process::Command, sync::Arc, time::Duration};

use crate::{Modifiers, RustileConfig, config::Action, server::state::{ClientState, RustileState}};
use calloop::{EventLoop, Interest, Mode, PostAction, generic::Generic};
use smithay::{
    backend::{
        input::{Event, InputEvent, KeyState, KeyboardKeyEvent},
        renderer::{Frame, Renderer, element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer, utils::draw_render_elements},
        winit::{self, WinitEvent},
    }, input::keyboard::FilterResult, utils::{Point, Rectangle, Transform}, wayland::{seat::WaylandFocus, socket::ListeningSocketSource}
};
use wayland_server::Display;

//En Wayland moderno, Calloop necesita acceso a ambos simultaneamente
//para despachar los mensajes correctamente
pub struct CalloopData {
    pub state: RustileState,
    pub display: Display<RustileState>,
}

pub struct RustileServer {
    pub config: RustileConfig,
}

impl RustileServer {
    pub fn new(config: RustileConfig) -> Self {
        Self { config }
    }

    pub fn set_config(&mut self, config: RustileConfig){
        self.config = config;
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Iniciando Rustile Wayland Compositor...");

        //Creamos el "Display" de Wayland (el servidor al que se conectan las apps)
        let mut display: Display<RustileState> = Display::new()?;

        //Creamos el estado de nuestro WM
        let state = RustileState::new(self.config, &mut display);

        //juntamos ambos en CalloopData
        let mut data = CalloopData { state, display };

        //Inicializamos el bucle de Eventos (Event Loop)
        //El tipo de dato que pasaremos en el bucle sera nuestro RustileState
        let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
        let loop_handle = event_loop.handle();

        //Inicializando el Backend Winit (Ventana de pruevas)
        println!("🖥️ Inicializando backend gráfico (Winit)...");
        let (mut backend, mut winit_loop) =
            winit::init::<GlesRenderer>().expect("Fallo al inicializar la ventana de Winit");

        //Smithay nos provee ListeingsSocketSource para reemplazar a bind_auto
        let socket_source = ListeningSocketSource::new_auto()?;
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
        println!(
            "✅ Servidor Wayland escuchando en el socket: {}",
            socket_name
        );

        //Exportamos la variable de entorno para que las apps sepan donde conectarse
        unsafe { std::env::set_var("WAYLAND_DISPLAY", &socket_name) };

        loop_handle.insert_source(socket_source, |client_stream, _, data| {
            // Cuando una app se conecta, le damos un estado vacío `Arc::new(())` por ahora
            if let Err(e) = data
                .display
                .handle()
                .insert_client(client_stream, Arc::new(ClientState::default()))
            {
                eprintln!("Error añadiendo cliente: {}", e);
            }
        })?;

        //Extraemos el "File Descriptor" del backend de Wayland y lo escuchamos manualmente
        let fd = data.display.backend().poll_fd().try_clone_to_owned()?;
        let display_source = Generic::new(fd, Interest::READ, Mode::Level);

        loop_handle.insert_source(display_source, |_, _, data| {
            data.display.dispatch_clients(&mut data.state).unwrap();
            Ok(PostAction::Continue)
        })?;

        //El bucle Infinito
        while data.state.is_running {
            winit_loop.dispatch_new_events(|event| {
                match event {
                    WinitEvent::Redraw => {
                        //Conectamos con la targeta grafia
                        backend.bind().unwrap();
                        let size = backend.window_size();

                        {
                            let mut guard = backend.bind().unwrap();


                            let mut elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();

                            for window in data.state.space.elements(){
                                //Bucamos sus coordenadas en el espacio
                                let location =  data.state.space.element_location(window).unwrap_or(Point::from((0,0)));
                                
                        
                                if let Some(surface) = window.wl_surface(){
                                    let window_elements = smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                                        &mut *guard.0, 
                                        &*surface, 
                                        location.to_physical(1), 
                                        1.0, 
                                        1.0, 
                                        smithay::backend::renderer::element::Kind::Unspecified
                                         );                             
                                    //Le pedimos a OpenGl que dibuje la ventana
                                    
                                    elements.extend(window_elements);
                                }

                                
                            
                            }

                            //Iniciamos un "Frame"
                            let mut frame = guard
                                .0
                                .render(&mut guard.1, size, Transform::Flipped180)
                                .unwrap();

                            //Limpiamos la pantalla con un color gris oscuro (RGBA)
                            let color = [0.5, 0.7, 0.1, 1.0];
                            let rect = Rectangle::from_size(size);
                            frame.clear(color.into(), &[rect]).unwrap();

                            let _ = draw_render_elements(&mut frame, 1.0, &elements, &[rect]).unwrap();

                            //Terminamos y enviamos el fotograma a la pantalla
                            let _ = frame.finish().unwrap();
                        }

                        backend.submit(None).unwrap();

                        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

                        for window in data.state.space.elements(){
                            window.send_frame(&data.state.output, time, Some(Duration::ZERO), |_, _| None);
                        }

                        backend.window().request_redraw();
                    }

                    WinitEvent::CloseRequested => {
                        println!("🛑 Cerrando Rustile de forma segura...");
                        println!("👋 Solicitud de cierre recibida.");
                        data.state.is_running = false;
                    }
                    WinitEvent::Input(InputEvent::Keyboard { event }) => {
                        
                            //Extraemos el codigo numerico de la tecla
                            let key_code: u32 = event.key_code().into();
                            let state_key = event.state();

                            let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                            let time = event.time_msec();

                            let keyboard = data.state.seat.get_keyboard().unwrap();
                            keyboard.input::<(), _>(
                                &mut data.state, 
                                event.key_code(), 
                                state_key, 
                                serial, 
                                time, 
                                |state, modifiers, keysym_handle|{
                                    if state_key == KeyState::Pressed {
                                        let keysyms = keysym_handle.modified_syms();
                                        println!("---------------------------------------------------");
                                        println!("⌨️ Tecla Bruta (Hardware): {:?}", key_code);
                                        println!("🔤 Tecla Traducida (Keysym): {:?}", keysyms);
                                        println!("🛡️ Modificadores: Super:{}, Alt:{}, Ctrl:{}, Shift:{}", modifiers.logo, modifiers.alt, modifiers.ctrl, modifiers.shift);
                                    }
                                    
                                    if state_key != KeyState::Pressed {
                                        return FilterResult::Forward;
                                    }

                                    let mut current_mod = Modifiers::NONE;
                                    if modifiers.alt { current_mod |= crate::config::Modifiers::ALT; }
                                    if modifiers.ctrl { current_mod |= crate::config::Modifiers::CTRL; }
                                    if modifiers.logo { current_mod |= crate::config::Modifiers::SUPER; }
                                    if modifiers.shift { current_mod |= crate::config::Modifiers::SHIFT; }

                                    let keysyms = keysym_handle.modified_syms();
                                    let mut action_to_run = None;

                                    for (shortcut, action) in &state.config.shortcuts{
                                        if current_mod == shortcut.modifier && keysyms.contains(&shortcut.key){
                                            action_to_run = Some(action.clone());
                                            break;
                                        }
                                    }
                                    
                                    if let Some(action) = action_to_run {
                                            match action {
                                                Action::Spawn(command) =>{
                                                    let wayland_socket = std::env::var("WAYLAND_DISPLAY")
                                                    .unwrap_or_else(|_| "wayland-1".to_string());
                                                println!("🚀 Ejecutando: {}", command);
                                                Command::new("sh")
                                                    .arg("-c")
                                                    .arg(command)
                                                    .env("WAYLAND_DISPLAY", wayland_socket)
                                                    .env_remove("DISPLAY")
                                                    .spawn()
                                                    .expect("Fallo al ejecutar");
                                                }
                                                Action::Quit => {
                                                    println!("🚪 Apagando Rustile...");
                                                    state.is_running = false;
                                                }
                                            }
                                            return FilterResult::Intercept(());
                                        }

                                     FilterResult::Forward
                                }
                            
                            );                        
                    }
                    _ => {}
                }
            });

            event_loop.dispatch(Some(Duration::from_millis(16)), &mut data)?;
            data.display.flush_clients()?;
        }

        println!("🛑 Rustile se ha cerrado correctamente.");
        Ok(())
    }
}
