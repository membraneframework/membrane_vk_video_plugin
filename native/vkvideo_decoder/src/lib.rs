use std::sync::Mutex;

use rustler::{Atom, Binary, Env, Error, NifStruct, OwnedBinary, ResourceArc, Term};
use vk_video::{parameters::DecoderParameters, BytesDecoder, EncodedInputChunk};

rustler::atoms! {
  ok,
  vk_instance_creation_failure,
  vk_adapter_creation_failure,
  vk_device_creation_failure,
  vk_decoder_creation_failure,
  owned_binary_allocation_failure,
  decoder_lock_failure,
  decode_failure,
  flush_failure
}

struct DecoderResource {
    pub decoder_mutex: Mutex<BytesDecoder>,
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(DecoderResource, env)
}

#[rustler::nif]
fn new() -> Result<(Atom, ResourceArc<DecoderResource>), Error> {
    let instance = vk_video::VulkanInstance::new()
        .map_err(|_| Error::Term(Box::new(vk_instance_creation_failure())))?;
    let adapter = instance
        .create_adapter(None)
        .map_err(|_| Error::Term(Box::new(vk_adapter_creation_failure())))?;
    let device = adapter
        .create_device(wgpu::Features::empty(), wgpu::Limits::default())
        .map_err(|_| Error::Term(Box::new(vk_device_creation_failure())))?;
    let decoder = device
        .create_bytes_decoder(DecoderParameters::default())
        .map_err(|_| Error::Term(Box::new(vk_decoder_creation_failure())))?;
    let decoder_mutex = Mutex::new(decoder);
    let resource = ResourceArc::new(DecoderResource { decoder_mutex });
    Ok((ok(), resource))
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.RawFrame"]
struct RawFrame<'a> {
    pub payload: Binary<'a>,
    pub pts_ns: Option<u64>,
    pub width: u32,
    pub height: u32,
}

#[rustler::nif(schedule = "DirtyCpu")]
fn decode<'a>(
    env: Env<'a>,
    resource: ResourceArc<DecoderResource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, Vec<RawFrame<'a>>), Error> {
    let mut decoder = resource
        .decoder_mutex
        .try_lock()
        .map_err(|err| Error::Term(Box::new((decoder_lock_failure(), err.to_string()))))?;
    let encoded_input_chunk = EncodedInputChunk {
        data: bytes.as_slice(),
        pts: pts_ns,
    };
    let decoded_frames = decoder
        .decode(encoded_input_chunk)
        .map_err(|err| Error::Term(Box::new((decode_failure(), err.to_string()))))?;
    let mut results = Vec::new();
    for frame in decoded_frames {
        let len = frame.data.frame.len();
        let mut payload = OwnedBinary::new(len)
            .ok_or(Error::Term(Box::new(owned_binary_allocation_failure())))?;
        payload.as_mut_slice().copy_from_slice(&frame.data.frame);

        results.push(RawFrame {
            payload: payload.release(env),
            pts_ns: frame.pts,
            width: frame.data.width,
            height: frame.data.height,
        });
    }
    Ok((ok(), results))
}
#[rustler::nif(schedule = "DirtyCpu")]
fn flush(env: Env, resource: ResourceArc<DecoderResource>) -> Result<(Atom, Vec<RawFrame>), Error> {
    let mut decoder = resource
        .decoder_mutex
        .try_lock()
        .map_err(|err| Error::Term(Box::new((decoder_lock_failure(), err.to_string()))))?;

    let flushed_frames = decoder
        .flush()
        .map_err(|err| Error::Term(Box::new((flush_failure(), err.to_string()))))?;

    let mut results = Vec::new();
    for frame in flushed_frames {
        let len = frame.data.frame.len();
        let mut payload = OwnedBinary::new(len)
            .ok_or(Error::Term(Box::new(owned_binary_allocation_failure())))?;
        payload.as_mut_slice().copy_from_slice(&frame.data.frame);

        results.push(RawFrame {
            payload: payload.release(env),
            pts_ns: frame.pts,
            width: frame.data.width,
            height: frame.data.height,
        });
    }
    Ok((ok(), results))
}

rustler::init!("Elixir.Membrane.VKVideo.Decoder.Native", load = load);
