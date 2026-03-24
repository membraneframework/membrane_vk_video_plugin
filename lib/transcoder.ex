defmodule Membrane.VKVideo.Transcoder do
  @moduledoc """
  H.264 hardware transcoder using Vulkan Video extensions.

  Accepts a single H.264 input stream and produces multiple independently
  configured H.264 output streams. Each output is described by a
  `Membrane.VKVideo.Transcoder.OutputSpec` entry in the `output_specs` option.

  Output pads are dynamic and are referenced as `Pad.ref(:output, index)`, where
  `index` is the zero-based position of the corresponding spec in `output_specs`.

  > #### Pad linking requirement {: .warning}
  >
  > All output pads (`Pad.ref(:output, 0)` through `Pad.ref(:output, N-1)`) **must** be
  > linked in the **same spec** in which the transcoder element itself is created.
  > Linking pads in a later spec is not supported.

  ## Example

      spec = [
        child(:source, source)
        |> child(:parser, %Membrane.H264.Parser{...})
        |> child(:transcoder, %Membrane.VKVideo.Transcoder{
          output_specs: [
            %Membrane.VKVideo.Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}},
            %Membrane.VKVideo.Transcoder.OutputSpec{width: 640, height: 360, frame_rate: {25, 1}}
          ]
        }),
        get_child(:transcoder) |> via_out(Pad.ref(:output, 0)) |> child(:sink_hd, sink_hd),
        get_child(:transcoder) |> via_out(Pad.ref(:output, 1)) |> child(:sink_sd, sink_sd)
      ]
  """

  use Membrane.Filter

  alias Membrane.Pad
  alias Membrane.VKVideo.{DeviceServer, Native}

  def_input_pad :input,
    accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}

  def_output_pad :output,
    availability: :on_request,
    accepted_format: %Membrane.H264{stream_structure: :annexb, alignment: :au}

  def_options output_specs: [
                spec: [Membrane.VKVideo.Transcoder.OutputSpec.t()],
                description: """
                List of output specifications. Each entry defines the target resolution,
                framerate, encoder tune, rate control, and scaling algorithm for one output
                stream. Output pads are referenced as `Pad.ref(:output, index)` where
                `index` is the zero-based position in this list.
                """
              ]

  @impl true
  def handle_init(_ctx, opts) do
    state = %{
      transcoder: nil,
      output_specs: opts.output_specs
    }

    {[], state}
  end

  @impl true
  def handle_setup(_ctx, state) do
    {:ok, device} = DeviceServer.get_device()
    {:ok, transcoder} = Native.new_transcoder(device, state.output_specs)
    {[], %{state | transcoder: transcoder}}
  end

  @impl true
  def handle_playing(ctx, state) do
    expected_pads =
      state.output_specs
      |> Enum.with_index()
      |> Enum.map(fn {_spec, idx} -> Pad.ref(:output, idx) end)

    output_pads =
      Map.keys(ctx.pads)
      |> Enum.filter(fn
        Pad.ref(:output, _id) -> true
        _other -> false
      end)

    missing_pads = expected_pads -- output_pads
    unexpected_pads = output_pads -- expected_pads

    if missing_pads != [] do
      raise """
      Not all output pads are linked. \
      Missing: #{inspect(missing_pads)}. \
      All expected output pads: #{inspect(expected_pads)} \
      must be linked in the same spec as the transcoder element.
      """
    end

    if unexpected_pads != [] do
      raise """
      Unexpected :output pads were linked: \
      #{inspect(unexpected_pads)}. \
      The only expected pads are: #{inspect(expected_pads)}.
      """
    end

    stream_format_actions =
      state.output_specs
      |> Enum.with_index()
      |> Enum.map(fn {spec, idx} ->
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
  def handle_pad_added(pad_ref, %{playback: :playing} = _ctx, _state) do
    raise """
    Unexpected :output pad was linked: \
    #{inspect(pad_ref)}
    """
  end

  @impl true
  def handle_pad_added(_pad_ref, _ctx, state) do
    {[], state}
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
      Enum.map(0..(length(state.output_specs) - 1), fn i ->
        {:end_of_stream, Pad.ref(:output, i)}
      end)

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
