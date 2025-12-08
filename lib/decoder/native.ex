defmodule Membrane.VKVideo.Decoder.Native do
  use Rustler, otp_app: :membrane_vk_video_plugin, crate: :vkvideo_decoder

  def new(), do: :erlang.nif_error(:nif_not_loaded)
  def decode(_decoder, _frame, _pts), do: :erlang.nif_error(:nif_not_loaded)
  def flush(_decoder), do: :erlang.nif_error(:nif_not_loaded)
end
