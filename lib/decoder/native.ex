defmodule Membrane.VKVideo.Decoder.Native do
  @moduledoc false
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_decoder

  @type t :: reference()
  @type raw_frame :: %{
          payload: binary(),
          pts: non_neg_integer() | nil,
          width: non_neg_integer(),
          height: non_neg_integer()
        }

  @spec new() :: {:ok, t()} | no_return()
  def new(), do: :erlang.nif_error(:nif_not_loaded)

  @spec decode(t(), binary(), pts_ns :: non_neg_integer() | nil) ::
          {:ok, raw_frame()} | no_return()
  def decode(_decoder, _frame, _pts \\ nil), do: :erlang.nif_error(:nif_not_loaded)

  @spec flush(t()) ::
          {:ok, raw_frame()} | no_return()
  def flush(_decoder), do: :erlang.nif_error(:nif_not_loaded)
end
