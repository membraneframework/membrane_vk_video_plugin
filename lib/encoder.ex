defmodule Membrane.VKVideo.Encoder do
  @moduledoc """
  H.264 encoder taking advantage of hardware acceleration provided
  by Vulkan video extensions.
  """
  use Membrane.Filter

  require Membrane.Logger

  alias __MODULE__.Native

  def_input_pad :input, accepted_format: %Membrane.RawVideo{pixel_format: :NV12}

  def_output_pad :output,
    accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}

  def_options tune: [
                spec: :low_latency | :high_quality,
                default: :low_latency,
                description: """
                Specifies whether the encoder should be optimized for minimal latency (which is
                important in case of livestreams) or for higher quality (applicable to offline encoding).
                """
              ],
              framerate: [
                spec: {non_neg_integer(), pos_integer()} | nil,
                default: nil,
                description: """
                Framerate of the stream expressed in number of frames per second.
                If nil, the framerate will be read from the stream format's structure.
                """
              ],
              rate_control: [
                spec:
                  :encoder_default
                  | :disabled
                  | Membrane.VKVideo.Encoder.VariableBitrate.t()
                  | Membrane.VKVideo.Encoder.ConstantBitrate.t(),
                default: :encoder_default,
                description: """
                Specifies which rate control mechanism should by used by the encoder.
                """
              ]

  @impl true
  def handle_init(_ctx, opts) do
    state = %{
      encoder: nil,
      width: nil,
      height: nil,
      override_framerate?: opts.framerate == nil,
      framerate: opts.framerate,
      rate_control: opts.rate_control
    }

    {[], state}
  end

  @impl true
  def handle_setup(_ctx, state) do
    {:ok, decoder} = Native.new(state.width, state.height, state.framerate, state.rate_control)
    state = %{state | decoder: decoder}
    {[], state}
  end

  @impl true
  def handle_stream_format(:input, stream_format, _ctx, %{override_framerate?: true} = state) do
    if stream_format.width != state.width or stream_format.height != state.height do
      if stream_format.framerate != nil and stream_format.framerate != state.framerate do
        Membrane.Logger.warning("""
        Framerate received within stream format: #{inspect(stream_format.framerate)} was overriden by the value provided via options:
        #{inspect(state.framerate)}
        """)
      end

      {:ok, encoder} = Native.new(state.width, state.height, state.framerate, state.rate_control)

      %{
        state
        | encoder: encoder,
          width: stream_format.width,
          height: stream_format.height
      }
      |> send_stream_format()
    else
      {[], state}
    end
  end

  @impl true
  def handle_stream_format(:input, stream_format, _ctx, %{override_framerate?: false} = state) do
    if stream_format.width != state.width or stream_format.height != state.height or
         stream_format.framerate != state.framerate do
      {:ok, encoder} = Native.new(state.width, state.height, state.framerate, state.rate_control)

      %{
        state
        | encoder: encoder,
          width: stream_format.width,
          height: stream_format.height,
          framerate: stream_format.framerate
      }
      |> send_stream_format()
    else
      {[], state}
    end
  end

  defp send_stream_format(state) do
    {[
       stream_format:
         {:output,
          %Membrane.H264{
            stream_structure: :annexb,
            alignment: :au,
            width: state.width,
            height: state.height,
            framerate: state.framerate
          }}
     ], state}
  end

  @impl true
  def handle_buffer(:input, buffer, _ctx, state) do
    {:ok, encoded_frames} = Native.encode(state.encoder, buffer.payload, buffer.pts)

    Enum.map(
      encoded_frames,
      &{:output,
       %Membrane.Buffer{
         payload: &1.payload,
         pts: Membrane.Time.nanoseconds(&1.pts_ns),
         dts: Membrane.Time.nanoseconds(&1.pts_ns)
       }}
    )
  end

  @impl true
  def handle_end_of_stream(:input, _ctx, state) do
    state = %{state | encoder: nil}
    {[end_of_stream: :output], state}
  end
end
