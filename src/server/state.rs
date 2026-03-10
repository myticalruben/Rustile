use smithay::{
    delegate_compositor, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, input::{SeatHandler, SeatState}, output::{Output, PhysicalProperties, Subpixel}, wayland::{
        buffer::BufferHandler, compositor::{CompositorClientState, CompositorHandler, CompositorState}, output::OutputHandler, shell::xdg::{XdgShellHandler, XdgShellState}, shm::{ShmHandler, ShmState}
    }
};
use wayland_server::{Display, backend::ClientData, protocol::wl_surface::WlSurface};

use crate::RustileConfig;

//Datos que guardamos por cada app (cliente) que se conecta
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

pub struct RustileState {
    pub config: RustileConfig,
    pub is_running: bool,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<RustileState>,
}

impl Default for ClientState{
    fn default() -> Self {
        Self { compositor_state: CompositorClientState::default() }
    }
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: wayland_server::backend::ClientId) {}
    fn disconnected(
        &self,
        _client_id: wayland_server::backend::ClientId,
        _reason: wayland_server::backend::DisconnectReason,
    ) {
    }
}

impl RustileState {
    pub fn new(config: RustileConfig, display: &mut Display<Self>) -> Self {
        let compositor_state = CompositorState::new::<Self>(&display.handle());
        let xdg_shell_state = XdgShellState::new::<Self>(&display.handle());
        let shm_state = ShmState::new::<Self>(&display.handle(), vec![]);
        let mut seat_state = SeatState::new();
        let _seat = seat_state.new_wl_seat(&display.handle(), "seat0");

        let output = Output::new("Rustile-1".into(), PhysicalProperties {
            size: (1920, 1080).into(),
            subpixel: Subpixel::Unknown,
            make: "Rustile".into(),
            model: "Monitor Virtual".into()
        });

        let _global = output.create_global::<Self>(&display.handle());

        Self {
            config,
            is_running: true,
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
        }
    }
}

// ==========================================================
// IMPLEMENTACIÓN DE LOS PROTOCOLOS (TRAITS)
// ==========================================================

impl OutputHandler for RustileState {}

impl SeatHandler for RustileState {
    type KeyboardFocus = WlSurface;

    type PointerFocus = WlSurface;

    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
}

impl CompositorHandler for RustileState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a wayland_server::Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, _surface: &wayland_server::protocol::wl_surface::WlSurface) {}
}

impl BufferHandler for RustileState {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}

impl ShmHandler for RustileState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for RustileState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        println!("🪟 ¡Nueva ventana solicitada!");

        surface.send_configure();
    }

    fn new_popup(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
    }

    fn grab(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _seat: wayland_server::protocol::wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {
    }

    fn reposition_request(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
        _token: u32,
    ) {
    }
}

// Usamos las macros de Smithay para conectar nuestros Traits con el servidor Wayland
delegate_shm!(RustileState);
delegate_seat!(RustileState);
delegate_output!(RustileState);
delegate_xdg_shell!(RustileState);
delegate_compositor!(RustileState);
