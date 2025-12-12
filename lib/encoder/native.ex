defmodule Membrane.VKVideo.Encoder.Native do
  @moduledoc false
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_encoder

  @type t :: reference()

  @spec new(
          non_neg_integer(),
          non_neg_integer(),
          {non_neg_integer(), non_neg_integer()},
          non_neg_integer() | nil
        ) :: {:ok, t()} | no_return()
  def new(_width, _height, _framerate, _average_bitrate \\ nil),
    do: :erlang.nif_error(:nif_not_loaded)

  @spec encode(t(), binary(), pts_ns :: non_neg_integer() | nil) ::
          :ok | {:error, String.t()} | no_return()
  def encode(_encoder, _raw_frame, _pts), do: :erlang.nif_error(:nif_not_loaded)
end
