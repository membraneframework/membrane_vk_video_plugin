defmodule Membrane.VKVideo.Encoder.Native do
  @moduledoc false
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_encoder

  @type t :: reference()

  @spec new(
          non_neg_integer(),
          non_neg_integer(),
          {non_neg_integer(), non_neg_integer()},
          :low_latency | :high_quality,
          non_neg_integer() | nil
        ) ::
          {:ok, t()}
          | {:error,
             {:vk_instance_creation_failure
              | :vk_adapter_creation_failure
              | :vk_device_creation_failure
              | :vk_decoder_creation_failure, String.t()}}
          | no_return()
  def new(_width, _height, _framerate, _tune \\ :low_latency, _average_bitrate \\ nil),
    do: :erlang.nif_error(:nif_not_loaded)

  @spec encode(t(), binary(), pts_ns :: non_neg_integer() | nil) ::
          :ok
          | {:error, :owned_binary_allocation_failure}
          | {:error, {:encoder_lock_failure | :encode_failure, String.t()}}
          | no_return()
  def encode(_encoder, _raw_frame, _pts \\ nil), do: :erlang.nif_error(:nif_not_loaded)
end
