use std::sync::Mutex;

use rustler::{Binary, Env, Error, NifStruct, ResourceArc, Term};
use vk_video::{parameters::DecoderParameters, BytesDecoder, EncodedInputChunk};

#[derive(NifStruct)]
#[module = "Elixir.VKVideo.DecoderOptions"]
struct DecoderOptions {}

struct DecoderResource {
    pub decoder_mutex: Mutex<BytesDecoder>,
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(DecoderResource, env);
    true
}

#[rustler::nif]
fn new() -> Result<ResourceArc<DecoderResource>, Error> {
    let instance = vk_video::VulkanInstance::new().unwrap();
    let adapter = instance.create_adapter(None).unwrap();

    let device = adapter
        .create_device(wgpu::Features::empty(), wgpu::Limits::default())
        .unwrap();

    let decoder = device
        .create_bytes_decoder(DecoderParameters::default())
        .unwrap();

    let decoder_mutex = Mutex::new(decoder);

    let resource = ResourceArc::new(DecoderResource { decoder_mutex });
    Ok(resource)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn decode(
    env: Env,
    resource: ResourceArc<DecoderResource>,
    bytes: Binary,
    pts: Option<u64>,
) -> Vec<Vec<u8>> {
    let mut decoder = resource.decoder_mutex.try_lock().unwrap();
    let encoded_input_chunk = EncodedInputChunk {
        data: bytes.as_slice(),
        pts,
    };
    let decoded_frames = decoder.decode(encoded_input_chunk).unwrap();

    let mut results = Vec::new();
    for frame in decoded_frames {
        results.push(frame.data.frame);
    }
    results
}

#[rustler::nif(schedule = "DirtyCpu")]
fn flush(env: Env, resource: ResourceArc<DecoderResource>) -> Vec<Vec<u8>> {
    let mut decoder = resource.decoder_mutex.try_lock().unwrap();

    let decoded_frames = decoder.flush();

    let mut results = Vec::new();
    for frame in decoded_frames {
        results.push(frame.data.frame);
    }
    results
}

rustler::init!("Elixir.Membrane.VKVideo.Decoder.Native", load = load);
