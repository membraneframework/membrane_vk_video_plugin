use rustler::{Atom, Error, NifTaggedEnum, ResourceArc};
use rustler::{Binary, Env, NifStruct, NifUnitEnum, OwnedBinary, Term};
use std::sync::Mutex;
use vk_video::parameters::{RateControl, Rational, VideoParameters};
use vk_video::{BytesEncoder, Frame, RawFrameData};
struct EncoderResource {
    pub encoder_mutex: Mutex<BytesEncoder>,
    pub width: u32,
    pub height: u32,
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(EncoderResource, env)
}

rustler::atoms! {
  ok
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.EncodedFrame"]
struct EncodedFrame<'a> {
    pub payload: Binary<'a>,
    pub pts_ns: Option<u64>,
}

#[derive(NifUnitEnum)]
enum EncoderTune {
    LowLatency,
    HighQuality,
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.Encoder.VariableBitrate"]
struct VariableBitrate {
    pub average_bitrate: u64,
    pub max_bitrate: u64,
    pub virtual_buffer_size_ms: u64,
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.Encoder.ConstantBitrate"]
struct ConstantBitrate {
    pub bitrate: u64,
    pub virtual_buffer_size_ms: u64,
}

#[derive(NifTaggedEnum)]
enum EncoderRateControl {
    EncoderDefault,
    VariableBitrate(VariableBitrate),
    ConstantBitrate(ConstantBitrate),
    Disabled,
}

impl Into<RateControl> for EncoderRateControl {
    fn into(self) -> RateControl {
        match self {
            EncoderRateControl::EncoderDefault => RateControl::EncoderDefault,
            EncoderRateControl::ConstantBitrate(config) => RateControl::ConstantBitrate {
                bitrate: config.bitrate,
                virtual_buffer_size: std::time::Duration::from_millis(
                    config.virtual_buffer_size_ms,
                ),
            },
            EncoderRateControl::VariableBitrate(config) => RateControl::VariableBitrate {
                average_bitrate: config.average_bitrate,
                max_bitrate: config.max_bitrate,
                virtual_buffer_size: std::time::Duration::from_millis(
                    config.virtual_buffer_size_ms,
                ),
            },

            EncoderRateControl::Disabled => RateControl::Disabled,
        }
    }
}

#[rustler::nif(schedule = "DirtyIo")]
fn new(
    width: u32,
    height: u32,
    frame_rate: (u32, u32),
    tune: EncoderTune,
    rate_control_option: Option<EncoderRateControl>,
) -> Result<(Atom, ResourceArc<EncoderResource>), Error> {
    let non_zero_width = std::num::NonZero::new(width).ok_or(Error::BadArg)?;
    let non_zero_height = std::num::NonZero::new(height).ok_or(Error::BadArg)?;
    let instance = vk_video::VulkanInstance::new()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let adapter = instance
        .create_adapter(None)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let device = adapter
        .create_device(wgpu::Features::empty(), wgpu::Limits::default())
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let video_parameters = VideoParameters {
        width: non_zero_width,
        height: non_zero_height,
        target_framerate: Rational {
            numerator: frame_rate.0,
            denominator: std::num::NonZero::new(frame_rate.1).ok_or(Error::BadArg)?,
        },
    };

    let rate_control = rate_control_option.unwrap_or(EncoderRateControl::EncoderDefault);

    let parameters = match tune {
        EncoderTune::LowLatency => device
            .encoder_parameters_low_latency(video_parameters, rate_control.into())
            .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?,
        EncoderTune::HighQuality => device
            .encoder_parameters_high_quality(video_parameters, rate_control.into())
            .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?,
    };

    let encoder = device
        .create_bytes_encoder(parameters)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let encoder_mutex = Mutex::new(encoder);
    let resource = ResourceArc::new(EncoderResource {
        encoder_mutex,
        width,
        height,
    });
    Ok((ok(), resource))
}

#[rustler::nif(schedule = "DirtyIo")]
fn encode<'a>(
    env: Env<'a>,
    resource: ResourceArc<EncoderResource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, EncodedFrame<'a>), Error> {
    let frame = Frame {
        data: RawFrameData {
            frame: bytes.to_vec(),
            width: resource.width,
            height: resource.height,
        },
        pts: pts_ns,
    };

    let mut encoder = resource
        .encoder_mutex
        .lock()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let encoded_frame = encoder
        .encode(&frame, false)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let len = encoded_frame.data.len();
    let mut payload =
        OwnedBinary::new(len).ok_or(Error::RaiseTerm(Box::new("Couldn't create OwnedBinary")))?;
    payload.as_mut_slice().copy_from_slice(&encoded_frame.data);

    Ok((
        ok(),
        EncodedFrame {
            payload: payload.release(env),
            pts_ns: encoded_frame.pts,
        },
    ))
}

rustler::init!("Elixir.Membrane.VKVideo.Encoder.Native", load = load);
