defmodule Decoder.Test do
  use ExUnit.Case, async: true
  import Membrane.ChildrenSpec
  import Membrane.Testing.Assertions
  alias Membrane.Testing.{Pipeline, Sink}

  @framerate_numerator 25
  @frame_duration_ms div(1000, @framerate_numerator)

  describe "Decoder decodes" do
    test "one hundred H.264 frames with B-frames in presentation order" do
      in_path = "./fixtures/input-100.h264" |> Path.expand(__DIR__)

      pid =
        Pipeline.start_link_supervised!(
          spec:
            child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
            |> child(:parser, %Membrane.H264.Parser{
              generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
            })
            |> child(:decoder, Membrane.VKVideo.Decoder)
            |> child(:sink, Sink)
        )

      assert_sink_playing(pid, :sink)

      assert_sink_stream_format(
        pid,
        :sink,
        %Membrane.RawVideo{width: 1280, height: 720, pixel_format: :I420, aligned: true}
      )

      Enum.each(0..99, fn i ->
        assert_sink_buffer(pid, :sink, buffer)
        assert buffer.pts == Membrane.Time.milliseconds(i * @frame_duration_ms)
      end)

      assert_end_of_stream(pid, :sink, :input, 10000)

      Pipeline.terminate(pid)
    end

    @tag :tmp_dir
    test "ten H.264 frames", ctx do
      in_path = "./fixtures/input-10.h264" |> Path.expand(__DIR__)
      out_path = Path.join(ctx.tmp_dir, "out.yuv")
      ref_path = "./fixtures/ref-10.yuv" |> Path.expand(__DIR__)

      pid =
        Pipeline.start_link_supervised!(
          spec:
            child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
            |> child(:parser, %Membrane.H264.Parser{
              generate_best_effort_timestamps: %{framerate: {@framerate_numerator, 1}}
            })
            |> child(:decoder, Membrane.VKVideo.Decoder)
            |> child(:sink, %Membrane.File.Sink{location: out_path})
        )

      assert_end_of_stream(pid, :sink, :input)
      Pipeline.terminate(pid)
      assert File.read!(out_path) == File.read!(ref_path)
    end
  end
end
