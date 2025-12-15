defmodule Encoder.NativeTest do
  use ExUnit.Case, async: true
  alias Membrane.VKVideo.Encoder.Native

  @width 1280
  @height 720
  # number of bytes per sample is 1.5 since we use 420 chroma subsampling
  @frame_size_in_bytes round(@width * @height * 1.5)
  @framerate {25, 1}

  @tag :encoder_native
  @tag :tmp_dir
  test "Encoder encodes raw frames in YUV format into H.264 stream", ctx do
    in_path = "./fixtures/ref-10.yuv" |> Path.expand(__DIR__)
    out_path = Path.join(ctx.tmp_dir, "out.h264")

    assert {:ok, file} = File.read(in_path)
    {:ok, encoder_ref} = Native.new(@width, @height, @framerate)
    raw_frames = for <<chunk::size(@frame_size_in_bytes)-binary <- file>>, do: chunk

    encoded_frames =
      Enum.map(raw_frames, fn raw_frame ->
        {:ok, encoded_frame} =
          Native.encode(encoder_ref, raw_frame)

        encoded_frame.payload
      end)

    stream = Enum.join(encoded_frames)
    File.write!(out_path, stream)
  end

  @tag :encoder_native
  @tag :tmp_dir
  test "Encoder encodes raw frames in YUV format into H.264 stream with desired framerate", ctx do
    in_path = "./fixtures/ref-10.yuv" |> Path.expand(__DIR__)
    out_path = Path.join(ctx.tmp_dir, "out.h264")

    assert {:ok, file} = File.read(in_path)
    rate_control = :disabled
    {:ok, encoder_ref} = Native.new(@width, @height, @framerate, :low_latency, rate_control)
    raw_frames = for <<chunk::size(@frame_size_in_bytes)-binary <- file>>, do: chunk

    encoded_frames =
      Enum.map(raw_frames, fn raw_frame ->
        {:ok, encoded_frame} =
          Native.encode(encoder_ref, raw_frame)

        encoded_frame.payload
      end)

    stream = Enum.join(encoded_frames)
    File.write!(out_path, stream)
  end
end
