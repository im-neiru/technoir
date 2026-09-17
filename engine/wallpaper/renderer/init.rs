use wgpu::{Adapter, Backend, Backends};

use crate::wallpaper::Screen;

impl super::WallpaperRenderer {
    pub(super) async fn select_adapter(
        instance: &wgpu::Instance,
        preferred: Option<PreferedDeviceKey>,
        screens: &[Screen],
    ) -> Option<Adapter> {
        if let Some(preferred) = preferred {
            for adapter in instance.enumerate_adapters(Backends::PRIMARY).await {
                let info = adapter.get_info();

                if info.name == preferred.name
                    && info.vendor == preferred.vendor_id
                    && info.device == preferred.device_id
                    && info.backend == preferred.backend
                {
                    return Some(adapter);
                }
            }
        }

        let mut best_adapter = None;
        let mut best_score = i32::MIN;

        for adapter in instance.enumerate_adapters(Backends::PRIMARY).await {
            let limits = adapter.limits();
            let info = adapter.get_info();

            let mut score = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => 5000,
                wgpu::DeviceType::IntegratedGpu => 3000,
                _ => 0,
            };

            score += (limits.max_compute_invocations_per_workgroup * 10) as i32;
            score += (limits.max_storage_buffer_binding_size / 1024 / 1024) as i32;

            let mut surface_score = 0;

            for screen in screens {
                if let Some(target) = &screen.target {
                    if !adapter.is_surface_supported(&target.surface.surface) {
                        surface_score = i32::MIN;
                        break;
                    }

                    surface_score += 1;
                }
            }

            score += surface_score;

            if score > best_score {
                best_score = score;
                best_adapter = Some(adapter);
            }
        }

        best_adapter
    }
}

pub struct PreferedDeviceKey {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub backend: Backend,
}
