use std::sync::Mutex;

use rustler::{Binary, Env, Error, NifStruct, OwnedBinary, ResourceArc, Term};
use vk_video::{parameters::DecoderParameters, BytesDecoder, EncodedInputChunk};

struct DecoderResource {
    pub decoder_mutex: Mutex<BytesDecoder>,
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(DecoderResource, env)
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

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.RawFrame"]
struct RawFrame<'a> {
    pub payload: Binary<'a>,
    pub pts: Option<u64>,
    pub width: u32,
    pub height: u32,
}

#[rustler::nif(schedule = "DirtyCpu")]
fn decode<'a>(
    env: Env<'a>,
    resource: ResourceArc<DecoderResource>,
    bytes: Binary,
    pts: Option<u64>,
) -> Result<Vec<RawFrame<'a>>, &'static str> {
    resource
        .decoder_mutex
        .try_lock()
        .map_err(|_err| "Couldn't obtain decoder lock")
        .and_then(|mut decoder| {
            let encoded_input_chunk = EncodedInputChunk {
                data: bytes.as_slice(),
                pts,
            };
            decoder
                .decode(encoded_input_chunk)
                .map_err(|_err| "Couldn't decode")
        })
        .map(|decoded_frames| {
            let mut results = Vec::new();
            for frame in decoded_frames {
                let len = frame.data.frame.len();
                let mut payload = OwnedBinary::new(len).unwrap();
                payload.as_mut_slice().copy_from_slice(&frame.data.frame);

                results.push(RawFrame {
                    payload: payload.release(env),
                    pts: frame.pts,
                    width: frame.data.width,
                    height: frame.data.height,
                });
            }
            results
        })
}
#[rustler::nif(schedule = "DirtyCpu")]
fn flush(env: Env, resource: ResourceArc<DecoderResource>) -> Result<Vec<RawFrame>, &'static str> {
    match resource.decoder_mutex.try_lock() {
        Ok(mut decoder) => {
            let decoded_frames = decoder.flush();

            let mut results = Vec::new();
            for frame in decoded_frames {
                let len = frame.data.frame.len();
                let mut payload = OwnedBinary::new(len).unwrap();
                payload.as_mut_slice().copy_from_slice(&frame.data.frame);

                results.push(RawFrame {
                    payload: payload.release(env),
                    pts: frame.pts,
                    width: frame.data.width,
                    height: frame.data.height,
                });
            }
            Ok(results)
        }
        Err(_e) => Err("Couldn't obtain decoder lock"),
    }
}

rustler::init!("Elixir.Membrane.VKVideo.Decoder.Native", load = load);
