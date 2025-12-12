defmodule Encoder.NativeTest do
  use ExUnit.Case, async: true
  alias Membrane.VKVideo.Encoder.Native

  @width 1280
  @height 720
  # number of bytes per sample is 12 since we use 420 chroma subsampling
  @frame_size_in_bytes @width * @height * 12
  @framerate {30, 1}

  @tag :encoder_native
  test "Encoder encodes raw frames in YUV format into H.264 stream" do
    in_path = "./fixtures/input-100.h264" |> Path.expand(__DIR__)

    assert {:ok, file} = File.read(in_path)
    {:ok, encoder_ref} = Native.new(@width, @height, @framerate)
    raw_frames = for <<chunk::size(@frame_size_in_bytes)-binary <- file>>, do: chunk

    encoded_frames =
      Enum.map(raw_frames, fn raw_frame ->
        {:ok, encoded_frame} = Encoder.encode(raw_frame)
        encoded_frame
      end)
  end
end
