use crate::ok;
use crate::Resource;
use rustler::{Atom, Binary, Env, Error, NifStruct, OwnedBinary, ResourceArc};
use std::sync::Mutex;
use vk_video::{parameters::DecoderParameters, BytesDecoder, EncodedInputChunk};

pub struct DecoderResource {
    pub decoder_mutex: Mutex<BytesDecoder>,
}

#[derive(NifStruct)]
#[module = "Membrane.VKVideo.RawFrame"]
pub struct RawFrame<'a> {
    pub payload: Binary<'a>,
    pub pts_ns: Option<u64>,
    pub width: u32,
    pub height: u32,
}

pub fn new(
    _env: Env,
    resource: ResourceArc<Resource>,
) -> Result<(Atom, ResourceArc<Resource>), Error> {
    let decoder = resource
        .device()
        .ok_or_else(|| Error::BadArg)?
        .device
        .create_bytes_decoder(DecoderParameters::default())
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let decoder_mutex = Mutex::new(decoder);
    let decoder = DecoderResource { decoder_mutex };
    let resource = ResourceArc::new(Resource::Decoder(decoder));
    Ok((ok(), resource))
}

pub fn decode<'a>(
    env: Env<'a>,
    resource: ResourceArc<Resource>,
    bytes: Binary,
    pts_ns: Option<u64>,
) -> Result<(Atom, Vec<RawFrame<'a>>), Error> {
    let mut decoder = resource
        .decoder()
        .ok_or_else(|| Error::BadArg)?
        .decoder_mutex
        .lock()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let encoded_input_chunk = EncodedInputChunk {
        data: bytes.as_slice(),
        pts: pts_ns,
    };
    let decoded_frames = decoder
        .decode(encoded_input_chunk)
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;
    let mut results = Vec::new();
    for frame in decoded_frames {
        let len = frame.data.frame.len();
        let mut payload = OwnedBinary::new(len)
            .ok_or(Error::RaiseTerm(Box::new("Couldn't create OwnedBinary")))?;
        payload.as_mut_slice().copy_from_slice(&frame.data.frame);

        results.push(RawFrame {
            payload: payload.release(env),
            pts_ns: frame.metadata.pts,
            width: frame.data.width,
            height: frame.data.height,
        });
    }
    Ok((ok(), results))
}

pub fn flush(env: Env, resource: ResourceArc<Resource>) -> Result<(Atom, Vec<RawFrame>), Error> {
    let mut decoder = resource
        .decoder()
        .ok_or_else(|| Error::BadArg)?
        .decoder_mutex
        .lock()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let flushed_frames = decoder
        .flush()
        .map_err(|err| Error::RaiseTerm(Box::new(err.to_string())))?;

    let mut results = Vec::new();
    for frame in flushed_frames {
        let len = frame.data.frame.len();
        let mut payload = OwnedBinary::new(len)
            .ok_or(Error::RaiseTerm(Box::new("Couldn't create OwnedBinary")))?;
        payload.as_mut_slice().copy_from_slice(&frame.data.frame);

        results.push(RawFrame {
            payload: payload.release(env),
            pts_ns: frame.metadata.pts,
            width: frame.data.width,
            height: frame.data.height,
        });
    }
    Ok((ok(), results))
}
