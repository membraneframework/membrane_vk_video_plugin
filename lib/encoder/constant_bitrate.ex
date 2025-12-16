defmodule Membrane.VKVideo.Encoder.ConstantBitrate do
  @moduledoc """
  Defines encoder setting for constant bitrate rate control algorithm.
  """

  @type t :: %__MODULE__{bitrate: non_neg_integer(), virtual_buffer_size_ms: non_neg_integer()}
  defstruct [:bitrate, :virtual_buffer_size_ms]
end
