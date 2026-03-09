mod state;

use std::{sync::Arc, time::Duration};

use crate::{RustileConfig, server::state::RustileState};
use calloop::{EventLoop, Interest, Mode, PostAction, generic::Generic};
use smithay::{
    backend::{
        renderer::{Frame, Renderer, gles::{GlesRenderer, GlesTarget}},
        winit::{self, WinitEvent},
    },
    utils::{Rectangle, Transform},
    wayland::socket::ListeningSocketSource,
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

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Iniciando Rustile Wayland Compositor...");

        //Creamos el "Display" de Wayland (el servidor al que se conectan las apps)
        let display: Display<RustileState> = Display::new()?;

        //Creamos el estado de nuestro WM
        let state = RustileState::new(self.config);

        //juntamos ambos en CalloopData
        let mut data = CalloopData { state, display };

        //Inicializamos el bucle de Eventos (Event Loop)
        //El tipo de dato que pasaremos en el bucle sera nuestro RustileState
        let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
        let loop_handle = event_loop.handle();

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
                .insert_client(client_stream, Arc::new(()))
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

        //Inicializando el Backend Winit (Ventana de pruevas)
        println!("🖥️ Inicializando backend gráfico (Winit)...");
        let (mut backend, mut winit_loop) =
            winit::init::<GlesRenderer>().expect("Fallo al inicializar la ventana de Winit");

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

                        //Iniciamos un "Frame"
                        let mut frame = guard.0.render(&mut guard.1, size, Transform::Normal).unwrap();
                        
                        //Limpiamos la pantalla con un color gris oscuro (RGBA)
                        let color = [0.1, 0.1, 0.1, 1.0];
                        let rect = Rectangle::from_size(size);
                        frame.clear(color.into(), &[rect]).unwrap();

                        //Terminamos y enviamos el fotograma a la pantalla
                        frame.finish().unwrap();
                        }

                        backend.submit(None).unwrap();
                        
                    }
                    WinitEvent::CloseRequested => {
                        println!("👋 Solicitud de cierre recibida.");
                        data.state.is_running = false;
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
