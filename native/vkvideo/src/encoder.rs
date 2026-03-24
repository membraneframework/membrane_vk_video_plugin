use crate::{ok, EncodedFrame, Resource};
use rustler::{Atom, Error, NifTaggedEnum, ResourceArc};
use rustler::{Binary, Env, NifStruct, NifUnitEnum, OwnedBinary};
use std::sync::Mutex;
use vk_video::parameters::{RateControl, Rational, VideoParameters};
use vk_video::{BytesEncoder, InputFrame, RawFrameData};

pub struct EncoderResource {
    pub encoder_mutex: Mutex<Option<BytesEncoder>>,
    pub width: u32,
    pub height: u32,
}

#[derive(NifUnitEnum, Clone, Copy)]
pub enum EncoderTune {
    LowLatency,
    HighQuality,
}

#[derive(NifStruct, Clone, Copy)]
#[module = "Membrane.VKVideo.Encoder.VariableBitrate"]
pub struct VariableBitrate {
    pub average_bitrate: u64,
    pub max_bitrate: u64,
    pub virtual_buffer_size_ms: u64,
}

#[derive(NifStruct, Clone, Copy)]
#[module = "Membrane.VKVideo.Encoder.ConstantBitrate"]
pub struct ConstantBitrate {
    pub bitrate: u64,
    pub virtual_buffer_size_ms: u64,
}

#[derive(NifTaggedEnum, Clone, Copy)]
pub enum EncoderRateControl {
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

pub fn new(
    _env: Env,
    resource: ResourceArc<Resource>,
    width: u32,
    height: u32,
    frame_rate: (u32, u32),
    tune: EncoderTune,
    rate_control: EncoderRateControl,
) -> Result<(Atom, ResourceArc<Resource>), Error> {
    let device_resource = &resource.device().ok_or_else(|| Error::BadArg)?.device;
    let non_zero_width = std::num::NonZero::new(width).ok_or(Error::BadArg)?;
    let non_zero_height = std::num::NonZero::new(height).ok_or(Error::BadArg)?;

    let video_parameters = VideoParameters {
        width: non_zero_width,
        height: non_zero_height,
        target_framerate: Rational {
            numerator: frame_rate.0,
            denominator: std::num::NonZero::new(frame_rate.1).ok_or(Error::BadArg)?,
        },
    };

    let parameters = match tune {
        EncoderTune::LowLatency => device_resource
            .encoder_parameters_low_latency(video_parameters, rate_control.into())
            .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?,
        EncoderTune::HighQuality => device_resource
            .encoder_parameters_high_quality(video_parameters, rate_control.into())
            .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?,
    };

    let encoder = device_resource
        .create_bytes_encoder(parameters)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let encoder_mutex = Mutex::new(Some(encoder));
    let encoder = EncoderResource {
        encoder_mutex,
        width,
        height,
    };

    let resource = ResourceArc::new(Resource::Encoder(encoder));
    Ok((ok(), resource))
}

pub fn encode<'a>(
    env: Env<'a>,
    resource: ResourceArc<Resource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, EncodedFrame<'a>), Error> {
    let encoder = resource.encoder().ok_or_else(|| Error::BadArg)?;
    let frame = InputFrame {
        data: RawFrameData {
            frame: bytes.to_vec(),
            width: encoder.width,
            height: encoder.height,
        },
        pts: pts_ns,
    };

    let mut guard = encoder
        .encoder_mutex
        .lock()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let encoder = guard.as_mut().ok_or(Error::BadArg)?;

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
