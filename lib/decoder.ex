defmodule Membrane.VKVideo.Decoder do
  use Membrane.Filter

  alias __MODULE__.Native

  def_input_pad :input, accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}
  def_output_pad :output, accepted_format: Membrane.RawVideo

  @impl true
  def handle_init(_ctx, _opts) do
    state = %{decoder: nil, width: nil, height: nil}
    {[], state}
  end

  @impl true
  def handle_setup(_ctx, state) do
    state = %{state | decoder: Native.new()}
    {[], state}
  end

  @impl true
  def handle_stream_format(:input, _stream_format, _ctx, state) do
    {[], state}
  end

  @impl true
  def handle_buffer(:input, buffer, _ctx, state) do
    {:ok, decoded_frames} = Native.decode(state.decoder, buffer.payload, buffer.pts)
    Enum.flat_map_reduce(decoded_frames, state, &prepare_actions(&1, &2))
  end

  defp prepare_actions(frame, state) do
    if frame.width != state.width and frame.height != state.height do
      actions = [
        stream_format:
          {:output,
           %Membrane.RawVideo{
             height: frame.height,
             width: frame.width,
             pixel_format: :I420,
             framerate: nil,
             aligned: true
           }},
        buffer:
          {:output, %Membrane.Buffer{payload: frame.payload, pts: frame.pts, dts: frame.pts}}
      ]

      state = %{state | width: frame.width, height: frame.height}
      {actions, state}
    else
      {[
         buffer:
           {:output, %Membrane.Buffer{payload: frame.payload, pts: frame.pts, dts: frame.pts}}
       ], state}
    end
  end

  @impl true
  def handle_end_of_stream(:input, _ctx, state) do
    {:ok, flushed_frames} = Native.flush(state.decoder)
    {actions, state} = Enum.flat_map_reduce(flushed_frames, state, &prepare_actions(&1, &2))
    {actions ++ [end_of_stream: :output], state}
  end
end
