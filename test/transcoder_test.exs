defmodule Transcoder.Test do
  use ExUnit.Case, async: false
  import Membrane.ChildrenSpec
  import Membrane.Testing.Assertions
  alias Membrane.{Pad, Testing}
  alias Membrane.Testing.{Pipeline, Sink}
  alias Membrane.VKVideo.Transcoder

  @framerate_numerator 25
  @frame_duration_ms div(1000, @framerate_numerator)

  describe "Transcoder" do
    @tag :requires_gpu
    test "sends correct stream format for each output pad" do
      in_path = "./fixtures/input-10.h264" |> Path.expand(__DIR__)

      output_specs = [
        %Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}},
        %Transcoder.OutputSpec{width: 640, height: 360, frame_rate: {25, 1}}
      ]

      pid =
        Pipeline.start_link_supervised!(
          spec: [
            child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
            |> child(:parser, %Membrane.H264.Parser{
              generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
            })
            |> child(:transcoder, %Transcoder{output_specs: output_specs}),
            get_child(:transcoder)
            |> via_out(Pad.ref(:output, 0))
            |> child(:sink_0, Sink),
            get_child(:transcoder)
            |> via_out(Pad.ref(:output, 1))
            |> child(:sink_1, Sink)
          ]
        )

      assert_sink_playing(pid, :sink_0)
      assert_sink_playing(pid, :sink_1)

      assert_sink_stream_format(pid, :sink_0, %Membrane.H264{
        width: 1280,
        height: 720,
        alignment: :au,
        stream_structure: :annexb,
        framerate: {25, 1}
      })

      assert_sink_stream_format(pid, :sink_1, %Membrane.H264{
        width: 640,
        height: 360,
        alignment: :au,
        stream_structure: :annexb,
        framerate: {25, 1}
      })

      assert_end_of_stream(pid, :sink_0, :input)
      assert_end_of_stream(pid, :sink_1, :input)
      Pipeline.terminate(pid)
    end

    @tag :requires_gpu
    test "produces buffers on all outputs and forwards end of stream" do
      in_path = "./fixtures/input-10.h264" |> Path.expand(__DIR__)

      output_specs = [
        %Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}},
        %Transcoder.OutputSpec{width: 640, height: 360, frame_rate: {25, 1}}
      ]

      pid =
        Pipeline.start_link_supervised!(
          spec: [
            child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
            |> child(:parser, %Membrane.H264.Parser{
              generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
            })
            |> child(:transcoder, %Transcoder{output_specs: output_specs}),
            get_child(:transcoder)
            |> via_out(Pad.ref(:output, 0))
            |> child(:sink_0, Sink),
            get_child(:transcoder)
            |> via_out(Pad.ref(:output, 1))
            |> child(:sink_1, Sink)
          ]
        )

      Enum.each(0..9, fn i ->
        assert_sink_buffer(pid, :sink_0, buffer_0)
        assert buffer_0.pts == Membrane.Time.milliseconds(i * @frame_duration_ms)

        assert_sink_buffer(pid, :sink_1, buffer_1)
        assert buffer_1.pts == Membrane.Time.milliseconds(i * @frame_duration_ms)
      end)

      assert_end_of_stream(pid, :sink_0, :input)
      assert_end_of_stream(pid, :sink_1, :input)
      Pipeline.terminate(pid)
    end

    @tag :requires_gpu
    test "works with a single output spec" do
      in_path = "./fixtures/input-10.h264" |> Path.expand(__DIR__)

      output_specs = [
        %Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}}
      ]

      pid =
        Pipeline.start_link_supervised!(
          spec: [
            child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
            |> child(:parser, %Membrane.H264.Parser{
              generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
            })
            |> child(:transcoder, %Transcoder{output_specs: output_specs}),
            get_child(:transcoder)
            |> via_out(Pad.ref(:output, 0))
            |> child(:sink_0, Sink)
          ]
        )

      assert_sink_stream_format(pid, :sink_0, %Membrane.H264{
        width: 1280,
        height: 720,
        alignment: :au,
        stream_structure: :annexb
      })

      assert_end_of_stream(pid, :sink_0, :input)
      Pipeline.terminate(pid)
    end

    test "raises when output pads are not all linked in the same spec" do
      in_path = "./fixtures/input-10.h264" |> Path.expand(__DIR__)

      output_specs = [
        %Transcoder.OutputSpec{width: 1280, height: 720, frame_rate: {25, 1}},
        %Transcoder.OutputSpec{width: 640, height: 360, frame_rate: {25, 1}}
      ]

      assert_raise RuntimeError, ~r/Missing.*output.*1/, fn ->
        pid =
          Pipeline.start_link_supervised!(
            spec: [
              child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
              |> child(:parser, %Membrane.H264.Parser{
                generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
              })
              |> child(:transcoder, %Transcoder{output_specs: output_specs}),
              # only pad 0 linked — pad 1 is missing
              get_child(:transcoder)
              |> via_out(Pad.ref(:output, 0))
              |> child(:sink_0, Sink)
            ]
          )

        assert_end_of_stream(pid, :sink_0, :input)
        Pipeline.terminate(pid)
      end
    end
  end
end
