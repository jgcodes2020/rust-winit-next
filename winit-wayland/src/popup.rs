use std::{marker::PhantomData, ops::Deref, sync::Arc};

use raw_window_handle::{HandleError, HasWindowHandle, WaylandWindowHandle, WindowHandle};
use raw_window_handle_05::HasRawWindowHandle as HasRawWindowHandle05;
use sctk::{compositor::CompositorState, shell::{self, xdg::{popup::{PopupConfigure, PopupHandler, Popup as XdgPopup}, XdgPositioner, XdgSurface}}};
use wayland_client::Dispatch;
use wayland_protocols::{wp::{fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1, viewporter::client::wp_viewport::WpViewport}, xdg::shell::client::xdg_positioner};
use winit_core::{application::Application, dpi::Size, window::{AnchorDirection, AnchorHints, PopupAttributes, Surface, Theme, WindowAttributes, WindowId}};

use crate::{event_loop::RuntimeState, state::WinitState};

pub struct Popup<T: Application + 'static> {
    /// The last received configure.
    pub last_configure: Option<PopupConfigure>,

    pub viewport: Option<WpViewport>,
    fractional_scale: Option<WpFractionalScaleV1>,

    /// The scale factor of the window.
    pub scale_factor: f64,

    /// Initial window size provided by the user. Removed on the first
    /// configure.
    initial_size: Option<Size>,

    compositor: Arc<CompositorState>,

    /// Whether the window is transparent.
    transparent: bool,

    pub redraw: bool,

    // Note, should be the last since it drops the surface.
    pub popup: XdgPopup,

    _phantom_tparam_do_not_use: PhantomData<T>
}

impl<T: Application + 'static> Popup<T> {
    pub fn new(winit: &mut WinitState<T>, parent: &impl XdgSurface, attributes: &PopupAttributes) -> Self {
        let compositor = winit.compositor.clone();
        let surface = compositor.create_surface(&winit.queue_handle);

        let positioner = {
            let positioner = XdgPositioner::new(&winit.xdg_shell).unwrap();

            let anchor_top_left = attributes.anchor_rect().0.to_physical::<i32>(1.0);
            let anchor_size = attributes.anchor_rect().1.to_physical::<i32>(1.0);
            positioner.set_anchor_rect(anchor_top_left.x, anchor_top_left.y, anchor_size.width, anchor_size.height);

            let inner_size = attributes.inner_size().to_physical::<i32>(1.0);
            positioner.set_size(inner_size.width, inner_size.height);

            positioner.set_gravity(to_xdg_positioner_gravity(attributes.surface_anchor()));
            positioner.set_anchor(to_xdg_positioner_anchor(attributes.rect_anchor()));

            positioner.set_constraint_adjustment(to_xdg_constraints(attributes.anchor_hints()).bits());

            positioner
        };

        let popup = XdgPopup::new(parent.xdg_surface(), &positioner.deref(), &winit.queue_handle, &*winit.compositor, &winit.xdg_shell).unwrap();
        let size = attributes.inner_size();

        Self {
            last_configure: None,
            viewport: None,
            fractional_scale: None,
            scale_factor: 1.0,
            initial_size: Some(size),
            compositor,
            transparent: true,
            redraw: false,
            popup,
            _phantom_tparam_do_not_use: PhantomData,
        }
    }

    pub fn configured(&self) -> bool {
        self.last_configure.is_some()
    }
}

impl<T: Application + 'static> Surface for Popup<T> {
    fn id(&self) -> WindowId {
        todo!()
    }

    fn scale_factor(&self) -> f64 {
        todo!()
    }

    fn request_redraw(&mut self) {
        todo!()
    }

    fn inner_size(&self) -> winit_core::dpi::PhysicalSize<u32> {
        todo!()
    }

    fn role(&self) -> winit_core::window::WindowRole {
        todo!()
    }

    fn role_mut(&mut self) -> winit_core::window::WindowRoleMut {
        todo!()
    }
}

impl<T: Application + 'static> HasWindowHandle for Popup<T> {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        todo!()
    }
}

unsafe impl<T: Application + 'static> HasRawWindowHandle05 for Popup<T> {
    fn raw_window_handle(&self) -> raw_window_handle_05::RawWindowHandle {
        todo!()
    }
}

impl<T: Application + 'static> PopupHandler for RuntimeState<T> {
    fn configure(
        &mut self,
        _: &wayland_client::Connection,
        queue_handle: &wayland_client::QueueHandle<Self>,
        popup: &XdgPopup,
        configure: PopupConfigure,
    ) {
        let winit = &mut self.winit;
        let window_id = crate::make_wid(popup.wl_surface());
    }

    fn done(&mut self, conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, popup: &XdgPopup) {
        // nothing
    }
}

impl<T: Application + 'static, U> Dispatch<xdg_positioner::XdgPositioner, U>  for RuntimeState<T> {
    fn event(
        state: &mut Self,
        proxy: &xdg_positioner::XdgPositioner,
        event: <xdg_positioner::XdgPositioner as wayland_client::Proxy>::Event,
        data: &U,
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // nothing, positioners do not generate events
    }
}

sctk::delegate_xdg_popup!(@<T: Application + 'static> RuntimeState<T>);

fn to_xdg_positioner_anchor(dir: AnchorDirection) -> xdg_positioner::Anchor {
    match dir {
        AnchorDirection::Center => xdg_positioner::Anchor::None,
        AnchorDirection::North => xdg_positioner::Anchor::Top,
        AnchorDirection::Northeast => xdg_positioner::Anchor::TopRight,
        AnchorDirection::East => xdg_positioner::Anchor::Right,
        AnchorDirection::Southeast => xdg_positioner::Anchor::BottomRight,
        AnchorDirection::South => xdg_positioner::Anchor::Bottom,
        AnchorDirection::Southwest => xdg_positioner::Anchor::BottomLeft,
        AnchorDirection::West => xdg_positioner::Anchor::Left,
        AnchorDirection::Northwest => xdg_positioner::Anchor::TopLeft,
    }
}
fn to_xdg_positioner_gravity(dir: AnchorDirection) -> xdg_positioner::Gravity {
    match dir {
        AnchorDirection::Center => xdg_positioner::Gravity::None,
        AnchorDirection::North => xdg_positioner::Gravity::Top,
        AnchorDirection::Northeast => xdg_positioner::Gravity::TopRight,
        AnchorDirection::East => xdg_positioner::Gravity::Right,
        AnchorDirection::Southeast => xdg_positioner::Gravity::BottomRight,
        AnchorDirection::South => xdg_positioner::Gravity::Bottom,
        AnchorDirection::Southwest => xdg_positioner::Gravity::BottomLeft,
        AnchorDirection::West => xdg_positioner::Gravity::Left,
        AnchorDirection::Northwest => xdg_positioner::Gravity::TopLeft,
    }
}
fn to_xdg_constraints(flags: AnchorHints) -> xdg_positioner::ConstraintAdjustment {
    let mut res = xdg_positioner::ConstraintAdjustment::None;
    if flags.contains(AnchorHints::SLIDE_X) {
        res |= xdg_positioner::ConstraintAdjustment::SlideX;
    }
    if flags.contains(AnchorHints::SLIDE_Y) {
        res |= xdg_positioner::ConstraintAdjustment::SlideY;
    }
    if flags.contains(AnchorHints::FLIP_X) {
        res |= xdg_positioner::ConstraintAdjustment::FlipX;
    }
    if flags.contains(AnchorHints::FLIP_Y) {
        res |= xdg_positioner::ConstraintAdjustment::FlipY;
    }
    if flags.contains(AnchorHints::RESIZE_X) {
        res |= xdg_positioner::ConstraintAdjustment::ResizeX;
    }
    if flags.contains(AnchorHints::RESIZE_Y) {
        res |= xdg_positioner::ConstraintAdjustment::ResizeY;
    }
    res
}