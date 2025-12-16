defmodule Membrane.VKVideo.Encoder.VariableBitrate do
  @moduledoc """
  Defines encoder setting for constant bitrate rate control algorithm.
  """

  @type t :: %__MODULE__{
          average_bitrate: non_neg_integer(),
          max_bitrate: non_neg_integer(),
          virtual_buffer_size_ms: non_neg_integer()
        }
  defstruct [:average_bitrate, :max_bitrate, :virtual_buffer_size_ms]
end
