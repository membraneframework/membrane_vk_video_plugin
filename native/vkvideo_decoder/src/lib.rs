use std::sync::Mutex;

use rustler::{Env, Error, ResourceArc, Term};

#[rustler::nif]
fn new(options: DecoderOptions) -> Result<ResourceArc<DecoderResource>, Error> {
    let instance = vk_video::VulkanInstance::new().unwrap();
    let surface = instance.wgpu_instance.create_surface(window).unwrap();
    let device = instance
        .create_device(
            wgpu::Features::empty(),
            wgpu::Limits::default(),
            Some(&surface),
        )
        .unwrap();

    let decoder = device.create_wgpu_textures_decoder().unwrap();

    let resource = ResourceArc::new(DecoderResource {
        instance,
        surface,
        device,
        decoder
    });
    return Ok(resource);
}


#[derive(NifStruct)]
#[module = "Elixir.VKVideo.DecoderOptions"]
struct Decoder {
    pub codec: String
}

struct DecoderResource {
    pub instance: Mutex<Decoder>,
    pub surface: Mutex<>,
    pub device: Mutex<>,
    pub decoder: Mutex<WgpuTexturesDecoder>
}


fn load(env: Env, _: Term) -> bool {
    rustler::resource!(DecoderResource, env);
    true
}

rustler::init!("Elixir.VKVideo.Decoder", load = load);
