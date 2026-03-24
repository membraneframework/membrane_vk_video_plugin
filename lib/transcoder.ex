defmodule Membrane.VKVideo.Transcoder do
  @moduledoc """
  H.264 hardware transcoder using Vulkan Video extensions.

  Accepts a single H.264 input stream and produces multiple independently
  configured H.264 output streams. Each output pad is configured via the
  `output_spec` pad option passed through `via_out/2`.

  Output pads are dynamic and are referenced as `Pad.ref(:output, index)`, where
  `index` must start at 0 and be consecutive (0, 1, 2, ...).

  > #### Pad linking requirement {: .warning}
  >
  > All output pads **must** be linked in the **same spec** in which the transcoder
  > element itself is created. Linking pads in a later spec is not supported.

  ## Example

      spec = [
        child(:source, source)
        |> child(:parser, %Membrane.H264.Parser{...})
        |> child(:transcoder, Membrane.VKVideo.Transcoder),
        get_child(:transcoder)
        |> via_out(Pad.ref(:output, 0), options: [output_spec: %Membrane.VKVideo.Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}}])
        |> child(:sink_hd, sink_hd),
        get_child(:transcoder)
        |> via_out(Pad.ref(:output, 1), options: [output_spec: %Membrane.VKVideo.Transcoder.OutputSpec{width: 640, height: 360, frame_rate: {25, 1}}])
        |> child(:sink_sd, sink_sd)
      ]
  """

  use Membrane.Filter

  alias Membrane.Pad
  alias Membrane.VKVideo.{DeviceServer, Native}

  def_input_pad :input,
    accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}

  def_output_pad :output,
    availability: :on_request,
    accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au},
    options: [
      output_spec: [
        spec: Membrane.VKVideo.Transcoder.OutputSpec.t(),
        description: """
        Output specification for this pad. Defines the target resolution,
        framerate, encoder tune, rate control, and scaling algorithm for the
        output stream.
        """
      ]
    ]

  @impl true
  def handle_init(_ctx, _opts) do
    state = %{
      transcoder: nil,
      output_specs: %{},
      device: nil
    }

    {[], state}
  end

  @impl true
  def handle_setup(_ctx, state) do
    {:ok, device} = DeviceServer.get_device()
    {[], %{state | device: device}}
  end

  @impl true
  def handle_pad_added(pad_ref, %{playback: :playing} = _ctx, _state) do
    raise """
    Output pad #{inspect(pad_ref)} was linked while the element is already playing. \
    All output pads must be linked in the same spec as the transcoder element.
    """
  end

  @impl true
  def handle_pad_added(Pad.ref(:output, idx), ctx, state) do
    spec = ctx.options.output_spec
    {[], %{state | output_specs: Map.put(state.output_specs, idx, spec)}}
  end

  @impl true
  def handle_playing(_ctx, state) do
    output_count = map_size(state.output_specs)
    expected_indices = MapSet.new(0..(output_count - 1)//1)
    actual_indices = MapSet.new(Map.keys(state.output_specs))

    if expected_indices != actual_indices do
      raise """
      Output pad indices must be consecutive starting from 0. \
      Got: #{inspect(actual_indices |> MapSet.to_list() |> Enum.sort())}
      """
    end

    ordered_specs =
      state.output_specs
      |> Enum.sort_by(fn {idx, _} -> idx end)
      |> Enum.map(fn {_, spec} -> spec end)

    {:ok, transcoder} = Native.new_transcoder(state.device, ordered_specs)
    state = %{state | transcoder: transcoder}

    stream_format_actions =
      state.output_specs
      |> Enum.sort_by(fn {idx, _} -> idx end)
      |> Enum.map(fn {idx, spec} ->
        {:stream_format,
         {Pad.ref(:output, idx),
          %Membrane.H264{
            stream_structure: :annexb,
            alignment: :au,
            width: spec.width,
            height: spec.height,
            framerate: spec.frame_rate
          }}}
      end)

    {stream_format_actions, state}
  end

  @impl true
  def handle_stream_format(:input, _stream_format, _ctx, state) do
    {[], state}
  end

  @impl true
  def handle_buffer(:input, buffer, _ctx, state) do
    {:ok, outputs} = Native.transcode(state.transcoder, buffer.payload, buffer.pts)
    {build_buffer_actions(outputs), state}
  end

  @impl true
  def handle_end_of_stream(:input, _ctx, state) do
    {:ok, flushed_outputs} = Native.flush_transcoder(state.transcoder)
    buffer_actions = build_buffer_actions(flushed_outputs)

    eos_actions =
      Map.keys(state.output_specs)
      |> Enum.sort()
      |> Enum.map(fn i -> {:end_of_stream, Pad.ref(:output, i)} end)

    {buffer_actions ++ eos_actions, %{state | transcoder: nil}}
  end

  defp build_buffer_actions(outputs) do
    Enum.flat_map(outputs, fn frame_per_pads ->
      Enum.with_index(frame_per_pads)
      |> Enum.map(fn {frame, idx} ->
        pad = Pad.ref(:output, idx)
        pts = if frame.pts_ns != nil, do: Membrane.Time.nanoseconds(frame.pts_ns), else: nil
        {:buffer, {pad, %Membrane.Buffer{payload: frame.payload, pts: pts}}}
      end)
    end)
  end
end
