use decoder::{DecoderResource, RawFrame};
use encoder::{EncodedFrame, EncoderRateControl, EncoderResource, EncoderTune};
use rustler::{Atom, Binary, Env, Error, ResourceArc, Term};

pub mod decoder;
pub mod encoder;

pub enum Resource {
    Encoder(EncoderResource),
    Decoder(DecoderResource),
}

impl Resource {
    pub fn encoder(&self) -> Option<&EncoderResource> {
        match self {
            Self::Encoder(encoder_resource) => Some(encoder_resource),
            Self::Decoder(_) => None,
        }
    }

    pub fn decoder(&self) -> Option<&DecoderResource> {
        match self {
            Self::Decoder(decoder_resource) => Some(decoder_resource),
            Self::Encoder(_) => None,
        }
    }
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(Resource, env)
}

#[rustler::nif(schedule = "DirtyIo")]
fn new_decoder() -> Result<(Atom, ResourceArc<Resource>), Error> {
    decoder::new()
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
    width: u32,
    height: u32,
    frame_rate: (u32, u32),
    tune: EncoderTune,
    rate_control: EncoderRateControl,
) -> Result<(Atom, ResourceArc<Resource>), Error> {
    encoder::new(width, height, frame_rate, tune, rate_control)
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
