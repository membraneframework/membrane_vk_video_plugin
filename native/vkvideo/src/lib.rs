use decoder::{DecoderResource, RawFrame};
use encoder::{EncoderRateControl, EncoderResource, EncoderTune};
use rustler::{Atom, Binary, Env, Error, ResourceArc, Term};
use std::sync::Arc;
use vk_video::{
    parameters::{VulkanAdapterDescriptor, VulkanDeviceDescriptor},
    VulkanDevice,
};

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
            _ => None,
        }
    }

    pub fn decoder(&self) -> Option<&DecoderResource> {
        match self {
            Self::Decoder(decoder_resource) => Some(decoder_resource),
            _ => None,
        }
    }

    pub fn device(&self) -> Option<&DeviceResource> {
        match self {
            Self::Device(device_resource) => Some(device_resource),
            _ => None,
        }
    }
}

#[derive(rustler::NifStruct)]
#[module = "Membrane.VKVideo.EncodedFrame"]
pub struct EncodedFrame<'a> {
    pub payload: Binary<'a>,
    pub pts_ns: Option<u64>,
}

#[allow(non_local_definitions)]
fn load(env: Env, _: Term) -> bool {
    rustler::resource!(Resource, env)
}

#[rustler::nif(schedule = "DirtyIo")]
fn create_device() -> Result<(Atom, ResourceArc<Resource>), Error> {
    let instance = vk_video::VulkanInstance::new()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let adapter_descriptor = VulkanAdapterDescriptor {
        supports_decoding: true,
        supports_encoding: true,
        compatible_surface: None,
    };
    let adapter = instance
        .create_adapter(&adapter_descriptor)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let device_descriptor = VulkanDeviceDescriptor {
        wgpu_features: wgpu::Features::empty(),
        wgpu_limits: wgpu::Limits::default(),
        wgpu_experimental_features: wgpu::ExperimentalFeatures::disabled(),
    };
    let device = adapter
        .create_device(&device_descriptor)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let device_resource = ResourceArc::new(Resource::Device(DeviceResource { device }));
    Ok((ok(), device_resource))
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

#[rustler::nif(schedule = "DirtyIo")]
fn destroy<'a>(env: Env<'a>, resource: ResourceArc<Resource>) -> Result<Atom, Error> {
    if let Resource::Encoder(encoder) = &*resource {
        let mut encoder = encoder
            .encoder_mutex
            .lock()
            .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
        *encoder = None;
    }

    Ok(ok())
}

// #[rustler::nif(schedule = "DirtyIo")]
// fn new_transcoder(
//     env: Env,
//     resource: ResourceArc<Resource>,
//     width: u32,
//     height: u32,
//     frame_rate: (u32, u32),
//     tune: EncoderTune,
//     rate_control: EncoderRateControl,
// ) -> Result<(Atom, ResourceArc<Resource>), Error> {
//     encoder::new(env, resource, width, height, frame_rate, tune, rate_control)
// }

// #[rustler::nif(schedule = "DirtyIo")]
// fn transcode<'a>(
//     env: Env<'a>,
//     resource: ResourceArc<Resource>,
//     bytes: Binary,
//     pts_ns: Option<u64>,
// ) -> Result<(Atom, EncodedFrame<'a>), Error> {
//     encoder::encode(env, resource, bytes, pts_ns)
// }

// #[rustler::nif(schedule = "DirtyIo")]
// pub fn flush_transcoder(
//     env: Env,
//     resource: ResourceArc<Resource>,
// ) -> Result<(Atom, Vec<RawFrame>), Error> {
//     transcoder::flush(env, resource)
// }

rustler::init!("Elixir.Membrane.VKVideo.Native", load = load);
