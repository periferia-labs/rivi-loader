use std::{borrow::BorrowMut, error::Error};

use ash::{Entry, extensions::ext::DebugUtils, version::{EntryV1_0, InstanceV1_0}, vk};

use crate::{compute::Compute, debug_layer::{DebugLayer, DebugOption}, gpu::GPU};


pub struct Vulkan {
    // Entry: Loads the Vulkan library.
    // Needs to outlive Instance and Device.
    _entry: ash::Entry,
    // Instance: Loads instance level functions.
    // Needs to outlive the Devices it has created.
    instance: ash::Instance,
    debug: Option<DebugLayer>,
}

impl Vulkan {

    pub(crate) unsafe fn new(
        debug: DebugOption
    ) -> Result<Vulkan, Box<dyn Error>> {

        let (layers, mut info) = debug.cons()?;
        let vk_layers: Vec<_> = layers
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let _entry = Entry::new()?;

        let instance = _entry
            .create_instance(&vk::InstanceCreateInfo::builder()
                .push_next(info.borrow_mut())
                .application_info(&vk::ApplicationInfo {
                    api_version: vk::make_version(1, 2, 0),
                    engine_version: 0,
                    ..Default::default()
                })
                .enabled_layer_names(&vk_layers)
                .enabled_extension_names(&[DebugUtils::name().as_ptr()])
            , None)?;

        println!("Instance created");
        match _entry.try_enumerate_instance_version()? {
            Some(v) => println!("Using Vulkan {}.{}.{}", vk::version_major(v), vk::version_minor(v), vk::version_patch(v)),
            None => println!("Using Vulkan 1.0"),
        };

        let debug_layer = DebugLayer::new(debug, &info, &_entry, &instance)?;
        Ok(Vulkan{_entry, instance, debug: debug_layer})
    }

    pub(crate) unsafe fn gpus(
        &self
    ) -> Result<Vec<GPU>, Box<dyn Error>> {
        let gpus = self.instance
            .enumerate_physical_devices()?
            .into_iter()
            .filter_map(|pdevice| GPU::new(&self.instance, pdevice))
            .collect::<Vec<GPU>>();

        match gpus.is_empty() {
            false => Ok(gpus),
            true => Err(format!("No compute capable GPUs"))?,
        }
    }

    pub(crate) unsafe fn logical_devices(
        &self,
        gpus: Vec<GPU>
    ) -> Vec<Compute> {
        gpus
            .iter()
            .filter_map(|gpu| match gpu.device(&self.instance) {
                Ok(gpu) => Some(gpu),
                Err(_) => None,
            })
            .collect::<Vec<Compute>>()
    }
}

impl Drop for Vulkan {
    fn drop(
        &mut self
    ) {
        println!("dropping app");
        if self.debug.is_some() {
            let debug = self.debug.as_ref().unwrap();
            unsafe { debug.loader.destroy_debug_utils_messenger(debug.callback, None) }
            println!("debug messenger destroyed");
        }
        unsafe { self.instance.destroy_instance(None) }
        println!("instance destroyed");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{debug_layer::DebugOption, new};

    #[test]
    fn app_new() {
        let init_timer = Instant::now();
        let res = new(DebugOption::None);
        assert!(res.is_ok());
        let (_app, devices) = res.unwrap();
        println!("Found {} logical device(s)", devices.len());
        println!("Found {} thread(s)", devices.iter().map(|f| f.fences.len()).sum::<usize>());
        println!("App new {}ms", init_timer.elapsed().as_millis());
    }

}