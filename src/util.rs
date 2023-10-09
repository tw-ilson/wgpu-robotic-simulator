pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
// pub fn make_cstring(source: &String) -> *const i8 {
//     (*source.as_bytes()).map(|u| u as i8)
// }
// pub fn sdl_gl_logstr(glstr: &[u8]) {
//     unsafe {
//         sdl2::log::log(&*format!("Vendor: {:?}", CStr::from_ptr(gl::GetString(gl::VENDOR).cast())));
//     }
// }
