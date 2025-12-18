use decoder::{DecoderResource, RawFrame};
use encoder::{EncodedFrame, EncoderRateControl, EncoderResource, EncoderTune};
use rustler::{Atom, Binary, Env, Error, ResourceArc, Term};
use std::sync::Arc;
use vk_video::VulkanDevice;

rustler::atoms! {
  ok,
}

pub mod decoder;
pub mod encoder;

pub enum Resource {
    Encoder(EncoderResource),
    Decoder(DecoderResource),
    Device(DeviceResource),
}

pub struct DeviceResource {
    pub device: Arc<VulkanDevice>,
}

impl std::panic::RefUnwindSafe for DeviceResource {}

impl Resource {
    pub fn encoder(&self) -> Option<&EncoderResource> {
        match self {
            Self::Encoder(encoder_resource) => Some(encoder_resource),
            Self::Decoder(_) => None,
            Self::Device(_) => None,
        }
    }

    pub fn decoder(&self) -> Option<&DecoderResource> {
        match self {
            Self::Decoder(decoder_resource) => Some(decoder_resource),
            Self::Encoder(_) => None,
            Self::Device(_) => None,
        }
    }

    pub fn device(&self) -> Option<&DeviceResource> {
        match self {
            Self::Device(device_resource) => Some(device_resource),
            Self::Encoder(_) => None,
            Self::Decoder(_) => None,
        }
    }
}

#[allow(non_local_definitions)]
fn load(env: Env, _: Term) -> bool {
    rustler::resource!(Resource, env)
}

#[rustler::nif(schedule = "DirtyIo")]
fn create_device() -> Result<ResourceArc<Resource>, Error> {
    let instance = vk_video::VulkanInstance::new()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let adapter = instance
        .create_adapter(None)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let device = adapter
        .create_device(wgpu::Features::empty(), wgpu::Limits::default())
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let device_resource = ResourceArc::new(Resource::Device(DeviceResource { device }));
    Ok(device_resource)
}

#[rustler::nif(schedule = "DirtyIo")]
fn new_decoder(
    env: Env,
    resource: ResourceArc<Resource>,
) -> Result<(Atom, ResourceArc<Resource>), Error> {
    decoder::new(env, resource)
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn decode<'a>(
    env: Env<'a>,
    resource: ResourceArc<Resource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, Vec<RawFrame<'a>>), Error> {
    decoder::decode(env, resource, bytes, pts_ns)
}

#[rustler::nif(schedule = "DirtyIo")]
pub fn flush_decoder(
    env: Env,
    resource: ResourceArc<Resource>,
) -> Result<(Atom, Vec<RawFrame>), Error> {
    decoder::flush(env, resource)
}

#[rustler::nif(schedule = "DirtyIo")]
fn new_encoder(
    env: Env,
    resource: ResourceArc<Resource>,
    width: u32,
    height: u32,
    frame_rate: (u32, u32),
    tune: EncoderTune,
    rate_control: EncoderRateControl,
) -> Result<(Atom, ResourceArc<Resource>), Error> {
    encoder::new(env, resource, width, height, frame_rate, tune, rate_control)
}

#[rustler::nif(schedule = "DirtyIo")]
fn encode<'a>(
    env: Env<'a>,
    resource: ResourceArc<Resource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, EncodedFrame<'a>), Error> {
    encoder::encode(env, resource, bytes, pts_ns)
}

rustler::init!("Elixir.Membrane.VKVideo.Native", load = load);
