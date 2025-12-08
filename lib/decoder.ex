defmodule Membrane.VKVideo.Decoder do
  use Membrane.Filter

  alias __MODULE__.Native


  def_input_pad :input, accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}
  def_output_pad :output, accepted_format: Membrane.RawVideo

  @impl true
  def handle_init(_ctx, _opts) do
    state = %{decoder: nil}
    {[], state}
  end

  @impl true
  def handle_setup(_ctx, state) do
    state = %{state | decoder: Native.new()}
    {[], state}
  end

  @impl true
  def handle_stream_format(:input, _stream_format, _ctx, state) do
    {[stream_format: {:output, %Membrane.RawVideo{height: 720, width: 1080, pixel_format: :I420,
    framerate: nil, aligned: true}}], state}
  end

  @impl true
  def handle_buffer(:input, buffer, _ctx, state) do
    decoded_buffers = Native.decode(state.decoder, buffer.payload, buffer.pts)
    actions = Enum.map(decoded_buffers, &{:buffer, {:output, %Membrane.Buffer{payload: &1}}})
    {actions, state}
  end
end
