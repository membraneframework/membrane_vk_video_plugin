defmodule Decoder.Test do
  use ExUnit.Case, async: true
  import Membrane.ChildrenSpec
  import Membrane.Testing.Assertions
  alias Membrane.Testing.{Pipeline, Sink}

  @tag :integration
  test "Decode" do
    in_path = "./fixtures/input-100.h264" |> Path.expand(__DIR__)

    pid =
      Pipeline.start_link_supervised!(
        spec:
          child(:file_src, %Membrane.File.Source{chunk_size: 40_960, location: in_path})
          |> child(:parser, %Membrane.H264.Parser{
            generate_best_effort_timestamps: %{framerate: {25, 1}}
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
      assert buffer.pts == Membrane.Time.milliseconds(i * 40)
    end)

    assert_end_of_stream(pid, :sink, :input, 10000)

    Pipeline.terminate(pid)
  end
end
