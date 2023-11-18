use imgui::{FontConfig, FontSource};
use imgui_winit_support::winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::{ptr, time::Instant};
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::Graphics::Direct3D9::{
    Direct3DCreate9, IDirect3D9, IDirect3DDevice9, D3DADAPTER_DEFAULT,
    D3DCREATE_SOFTWARE_VERTEXPROCESSING, D3DDEVTYPE_HAL, D3DFMT_R5G6B5, D3DMULTISAMPLE_NONE,
    D3DPRESENT_INTERVAL_DEFAULT, D3DPRESENT_PARAMETERS, D3DPRESENT_RATE_DEFAULT,
    D3DSWAPEFFECT_DISCARD, D3D_SDK_VERSION,
};
use windows::Win32::System::SystemServices::D3DCLEAR_TARGET;

const WINDOW_WIDTH: f64 = 1280.0;
const WINDOW_HEIGHT: f64 = 800.0;
const WINDOW_TITLE: &str = "まっぷえでぃた(v0.1)";
const WINDOW_BACK_CLEAR_COLOR: u32 = 0x0000;
unsafe fn set_up_dx_context(hwnd: HWND) -> (IDirect3D9, IDirect3DDevice9) {
    let d9_option = Direct3DCreate9(D3D_SDK_VERSION);
    match d9_option {
        Some(d9) => {
            let mut present_params = D3DPRESENT_PARAMETERS {
                BackBufferCount: 1,
                MultiSampleType: D3DMULTISAMPLE_NONE,
                MultiSampleQuality: 0,
                SwapEffect: D3DSWAPEFFECT_DISCARD,
                hDeviceWindow: hwnd,
                Flags: 0,
                FullScreen_RefreshRateInHz: D3DPRESENT_RATE_DEFAULT,
                PresentationInterval: D3DPRESENT_INTERVAL_DEFAULT as u32,
                BackBufferFormat: D3DFMT_R5G6B5,
                EnableAutoDepthStencil: BOOL(0),
                Windowed: BOOL(1),
                BackBufferWidth: WINDOW_WIDTH as _,
                BackBufferHeight: WINDOW_HEIGHT as _,
                ..core::mem::zeroed()
            };
            let mut device: Option<IDirect3DDevice9> = None;
            match d9.CreateDevice(
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_HAL,
                hwnd,
                D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32,
                &mut present_params,
                &mut device,
            ) {
                Ok(_) => (d9, device.unwrap()),
                _ => panic!("CreateDevice failed"),
            }
        }
        None => panic!("Direct3DCreate9 failed"),
    }
}
fn show_main_tab(ui: &imgui::Ui) {
    let mut opened = true;
    ui.main_menu_bar(|| {
        ui.menu("FILE", || {
            if ui.menu_item("save") {}
            if ui.menu_item("load") {}
        });
        ui.menu("EDIT", || {
            if ui.menu_item("new-map") {
                show_edit_window(&ui, &mut opened);
            }
        });

        ui.menu("VIEW", || if ui.menu_item("config") {});
        ui.menu("TOOLS", || {});
        ui.menu("LAYER", || {});
        ui.menu("WINDOW", || {});
    });
}
fn show_edit_window(ui: &imgui::Ui, opened: &mut bool) {
    ui.window("EDIT")
        .size([500.0, 200.0], imgui::Condition::FirstUseEver)
        .opened(opened)
        .build(|| {});
}
fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_resizable(false)
        .with_inner_size(LogicalSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    //let hwnd = if let RawWindowHandle::Win32(handle) =window.raw_window_handle() {
    let hwnd = HWND(isize::from(window.hwnd()));
    //} else {
    //unreachable!()
    //};

    let (_d9, device) = unsafe { set_up_dx_context(hwnd) };
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../../imgui-dx9-renderer/PixelMplus12-Regular.ttf"),
        size_pixels: 20.0,
        config: None,
    }]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer =
        unsafe { imgui_dx9_renderer::Renderer::new(&mut imgui, device.clone()).unwrap() };
    //imgui_wgpu::Texture::new(&device);
    let mut tex = renderer.textures();
    let mut last_frame = Instant::now();
    let mut edit_window_opened = false;

    event_loop.run(move |event, _, control_flow| {
        //let control_flow:&mut winit::event_loop::ControlFlow = control_flow;
        *control_flow = imgui_winit_support::winit::event_loop::ControlFlow::Poll;
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let io = imgui.io_mut();
                platform
                    .prepare_frame(io, &window)
                    .expect("Failed to start frame");
                window.request_redraw();
            }

            Event::MainEventsCleared => {
                let io = imgui.io_mut();
                platform
                    .prepare_frame(io, &window)
                    .expect("Failed to start frame");
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                unsafe {
                    device
                        .Clear(
                            0,
                            ptr::null_mut(),
                            D3DCLEAR_TARGET as u32,
                            WINDOW_BACK_CLEAR_COLOR,
                            1.0,
                            0,
                        )
                        .unwrap();
                    device.BeginScene().unwrap();
                }
                // メイン描画
                let ui = imgui.new_frame();
                if ui.is_key_pressed(imgui::Key::Escape) 
                {
                    *control_flow = imgui_winit_support::winit::event_loop::ControlFlow::Exit;
                }
                show_main_tab(&ui);
                renderer.render(imgui.render()).unwrap();
                unsafe {
                    device.EndScene().unwrap();
                    device
                        .Present(ptr::null_mut(), ptr::null_mut(), None, ptr::null_mut())
                        .unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = imgui_winit_support::winit::event_loop::ControlFlow::Exit;
            }
            event => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            _ => {}
        }
    });
}
