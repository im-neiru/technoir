use core::ptr::{self, NonNull};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

pub(crate) fn run() {
    let instance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
        .expect("Failed to retrieve module handle");

    println!("{instance:#?}")
}
