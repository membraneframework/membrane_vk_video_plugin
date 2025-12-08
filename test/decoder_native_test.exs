defmodule Decoder.NativeTest do
  use ExUnit.Case, async: true
  alias Membrane.VKVideo.Decoder.Native

  test "Decode 1 240p frame" do
    in_path = "./fixtures/input-100.h264" |> Path.expand(__DIR__)

    assert {:ok, file} = File.read(in_path)
    assert decoder_ref = Native.new()
    results = Native.decode(decoder_ref, file, 0)
    IO.inspect(length(results))
    results = Native.flush(decoder_ref)
    IO.inspect(length(results))
  end
end
