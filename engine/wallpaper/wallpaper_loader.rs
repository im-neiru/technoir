use super::{CleanupContext, PrepareContext, Screen, WallpaperProvider};
use indexmap::IndexMap;
use libloading::{Library, Symbol};
use std::path::Path;

pub struct WallpaperLoader {
    libraries: IndexMap<String, Library>,
}

impl WallpaperLoader {
    pub fn new() -> Self {
        Self {
            libraries: IndexMap::new(),
        }
    }

    pub fn load(
        &mut self,
        screen: &mut Screen,
        path: impl AsRef<Path>,
        prepare_context: &mut PrepareContext,
        cleanup_context: &mut CleanupContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.remove(screen, cleanup_context);

        let path_str = path.as_ref().to_string_lossy().to_string();

        let (lib_index, lib) = if let Some(index) = self.libraries.get_index_of(&path_str) {
            (index, &self.libraries[index])
        } else {
            let lib = unsafe { Library::new(path.as_ref())? };
            let (index, _) = self.libraries.insert_full(path_str, lib);
            (index, &self.libraries[index])
        };

        let provider = unsafe {
            #[allow(clippy::type_complexity)]
            let constructor: Symbol<
                unsafe extern "Rust" fn(*mut PrepareContext) -> Box<dyn WallpaperProvider>,
            > = lib.get(b"create\0")?;
            constructor(prepare_context as *mut _)
        };

        screen.provider = Some(provider);
        screen.library_index = Some(lib_index);

        Ok(())
    }

    pub fn remove(&mut self, screen: &mut Screen, cleanup_context: &mut CleanupContext) {
        if let Some(provider) = screen.provider.take() {
            provider.clean_up(cleanup_context);
        }
        screen.library_index = None;
    }

    pub fn swap(&mut self, screen_a: &mut Screen, screen_b: &mut Screen) {
        std::mem::swap(&mut screen_a.provider, &mut screen_b.provider);
        std::mem::swap(&mut screen_a.library_index, &mut screen_b.library_index);
    }

    pub fn get_provider<'a>(&self, screen: &'a Screen) -> Option<&'a dyn WallpaperProvider> {
        screen.provider.as_deref()
    }

    pub fn clear(&mut self, screens: &mut [Screen], cleanup_context: &mut CleanupContext) {
        for screen in screens {
            self.remove(screen, cleanup_context);
        }
    }
}

impl Default for WallpaperLoader {
    fn default() -> Self {
        Self::new()
    }
}
