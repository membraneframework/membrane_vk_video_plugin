use rustler::{Atom, Encoder, Error, ResourceArc};
use rustler::{Binary, Env, NifStruct, OwnedBinary, Term};
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
  ok,
  vk_instance_creation_failure,
  vk_adapter_creation_failure,
  vk_device_creation_failure,
  vk_decoder_creation_failure,
  encoder_parameters_creation_error,
  encoder_lock_failure,
  encode_failure,
  flush_failure
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.EncodedFrame"]
struct EncodedFrame<'a> {
    pub payload: Binary<'a>,
    pub pts_ns: Option<u64>,
}

#[rustler::nif]
fn new(
    width: u32,
    height: u32,
    frame_rate: (u32, u32),
    average_bitrate_option: Option<u64>,
) -> Result<(Atom, ResourceArc<EncoderResource>), Error> {
    let non_zero_width = std::num::NonZero::new(width).ok_or(Error::BadArg)?;
    let non_zero_height = std::num::NonZero::new(height).ok_or(Error::BadArg)?;
    let instance = vk_video::VulkanInstance::new()
        .map_err(|_| Error::Term(Box::new(vk_instance_creation_failure())))?;
    let adapter = instance
        .create_adapter(None)
        .map_err(|_| Error::Term(Box::new(vk_adapter_creation_failure())))?;
    let device = adapter
        .create_device(wgpu::Features::empty(), wgpu::Limits::default())
        .map_err(|_| Error::Term(Box::new(vk_device_creation_failure())))?;

    let video_parameters = VideoParameters {
        width: non_zero_width,
        height: non_zero_height,
        target_framerate: Rational {
            numerator: frame_rate.0,
            denominator: std::num::NonZero::new(frame_rate.1).ok_or(Error::BadArg)?,
        },
    };

    let average_bitrate = average_bitrate_option.unwrap_or(1_000_000);
    let rate_control = RateControl::VariableBitrate {
        average_bitrate,
        max_bitrate: average_bitrate * 2,
        virtual_buffer_size: std::time::Duration::from_secs(2),
    };
    let encoder = device
        .create_bytes_encoder(
            device
                .encoder_parameters_low_latency(video_parameters, rate_control)
                .map_err(|_| Error::Term(Box::new(encoder_parameters_creation_error())))?,
        )
        .map_err(|_| Error::Term(Box::new(vk_decoder_creation_failure())))?;
    let encoder_mutex = Mutex::new(encoder);
    let resource = ResourceArc::new(EncoderResource {
        encoder_mutex,
        width,
        height,
    });
    Ok((ok(), resource))
}

#[rustler::nif]
fn encode<'a>(
    env: Env<'a>,
    resource: ResourceArc<EncoderResource>,
    bytes: Binary,
    pts: Option<u64>,
) -> Result<(Atom, EncodedFrame<'a>), Error> {
    let frame = Frame {
        data: RawFrameData {
            frame: bytes.to_vec(),
            width: resource.width,
            height: resource.height,
        },
        pts,
    };

    let mut encoder = resource
        .encoder_mutex
        .try_lock()
        .map_err(|_| Error::Term(Box::new(encoder_lock_failure())))?;

    let encoded_frames = encoder
        .encode(&frame, false)
        .map_err(|_| Error::Term(Box::new(encode_failure())))?;

    let len = frame.data.frame.len();
    let mut payload = OwnedBinary::new(len).unwrap();
    payload.as_mut_slice().copy_from_slice(&frame.data.frame);

    Ok((
        ok(),
        EncodedFrame {
            payload: payload.release(env),
            pts_ns: frame.pts,
        },
    ))
}

rustler::init!("Elixir.Membrane.VKVideo.Encoder", load = load);
