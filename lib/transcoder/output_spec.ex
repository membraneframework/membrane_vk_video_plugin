defmodule Membrane.VKVideo.Transcoder.OutputSpec do
  @moduledoc """
  Defines a single output specification for the `Membrane.VKVideo.Transcoder` element.

  Each output spec corresponds to one output pad (`Pad.ref(:output, index)`) where the index
  matches the position of the spec in the `output_specs` list.

  Fields:
  * `width` - output frame width in pixels
  * `height` - output frame height in pixels
  * `tune` - encoder tuning preset: `:low_latency` (default) or `:high_quality`
  * `rate_control` - rate control mode; see `Membrane.VKVideo.Encoder` for available options
  * `scaling_algorithm` - algorithm used when scaling the input to the output resolution:
    `:nearest_neighbor`, `:lanczos3`, or `:bilinear` (default)
  """

  alias Membrane.VKVideo.Encoder

  @type t :: %__MODULE__{
          width: non_neg_integer(),
          height: non_neg_integer(),
          tune: :low_latency | :high_quality,
          rate_control:
            :encoder_default
            | :disabled
            | {:variable_bitrate, Encoder.VariableBitrate.t()}
            | {:constant_bitrate, Encoder.ConstantBitrate.t()},
          scaling_algorithm: :nearest_neighbor | :lanczos3 | :bilinear
        }

  @enforce_keys [:width, :height]
  defstruct @enforce_keys ++
              [
                tune: :low_latency,
                rate_control: :encoder_default,
                scaling_algorithm: :bilinear
              ]
end
