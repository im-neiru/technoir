#[no_mangle]
pub extern "C" fn render_frame(time: f32) {
    println!("Rendering from DLL at time {}", time);
}
