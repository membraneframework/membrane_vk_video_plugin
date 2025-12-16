defmodule Encoder.Test do
  use ExUnit.Case, async: true
  import Membrane.ChildrenSpec
  import Membrane.Testing.Assertions
  alias Membrane.Testing.{Pipeline, Sink}

  @width 1280
  @height 720
  # number of bytes per sample is 1.5 since we use 420 chroma subsampling
  @frame_size_in_bytes round(@width * @height * 1.5)
  @framerate_numerator 25
  @frame_duration_ms div(1000, @framerate_numerator)

  defmodule StreamFormatSwitcher do
    use Membrane.Filter
    def_input_pad :input, accepted_format: _any
    def_output_pad :output, accepted_format: %Membrane.RawVideo{}
    def_options width: [], height: [], framerate: [default: nil]

    @impl true
    def handle_playing(_ctx, state) do
      {[
         stream_format:
           {:output,
            %Membrane.RawVideo{
              width: state.width,
              height: state.height,
              aligned: true,
              pixel_format: :NV12,
              framerate: state.framerate
            }}
       ], state}
    end

    @impl true
    def handle_stream_format(:input, _stream_format, _ctx, state) do
      {[], state}
    end

    @impl true
    def handle_buffer(:input, buffer, _ctx, state) do
      {[buffer: {:output, buffer}], state}
    end
  end

  describe "Encoder encodes" do
    @tag :requires_gpu
    test "raw video frames" do
      in_path = "./fixtures/ref-10.nv12" |> Path.expand(__DIR__)

      pid =
        Pipeline.start_link_supervised!(
          spec:
            child(:file_src, %Membrane.File.Source{
              chunk_size: @frame_size_in_bytes,
              location: in_path
            })
            |> child(:stream_format_switcher, %StreamFormatSwitcher{
              width: @width,
              height: @height
            })
            |> child(:encoder, %Membrane.VKVideo.Encoder{framerate: {@framerate_numerator, 1}})
            |> child(:sink, Sink)
        )

      assert_sink_playing(pid, :sink)

      assert_sink_stream_format(
        pid,
        :sink,
        %Membrane.H264{width: 1280, height: 720, alignment: :au, stream_structure: :annexb}
      )

      assert_end_of_stream(pid, :sink, :input)

      Pipeline.terminate(pid)
    end
  end
end
