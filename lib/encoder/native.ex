defmodule Membrane.VKVideo.Encoder.Native do
  @moduledoc false
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_encoder

  @type t :: reference()

  @type encoded_frame :: %{
          payload: binary(),
          pts: non_neg_integer() | nil
        }

  @spec new(
          non_neg_integer(),
          non_neg_integer(),
          {non_neg_integer(), non_neg_integer()},
          :low_latency | :high_quality,
          :encoder_default
          | :disabled
          | {:variable_bitrate, Membrane.VKVideo.Encoder.VariableBitrate.t()}
          | {:constant_bitrate, Membrane.VKVideo.Encoder.ConstantBitrate.t()}
        ) :: {:ok, t()} | no_return()
  def new(_width, _height, _framerate, _tune, _rate_control),
    do: :erlang.nif_error(:nif_not_loaded)

  @spec encode(t(), binary(), pts_ns :: non_neg_integer() | nil) ::
          {:ok, encoded_frame()} | no_return()
  def encode(_encoder, _raw_frame, _pts \\ nil), do: :erlang.nif_error(:nif_not_loaded)
end
