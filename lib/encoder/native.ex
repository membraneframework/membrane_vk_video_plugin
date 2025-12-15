defmodule Membrane.VKVideo.Encoder.Native do
  @moduledoc false
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_encoder

  @type t :: reference()

  @spec new(
          non_neg_integer(),
          non_neg_integer(),
          {non_neg_integer(), non_neg_integer()},
          :low_latency | :high_quality,
          :encoder_default
          | :disabled
          | Membrane.VKVideo.Encoder.VariableBitrate.t()
          | Membrane.VKVideo.Encoder.ConstantBitrate.t()
        ) :: {:ok, t()} | no_return()
  def new(_width, _height, _framerate, _tune \\ :low_latency, _average_bitrate \\ nil),
    do: :erlang.nif_error(:nif_not_loaded)

  @spec encode(t(), binary(), pts_ns :: non_neg_integer() | nil) :: :ok | no_return()
  def encode(_encoder, _raw_frame, _pts \\ nil), do: :erlang.nif_error(:nif_not_loaded)
end
