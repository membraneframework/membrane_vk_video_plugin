defmodule Membrane.VKVideo do
  def start_link() do
    GenServer.start_link(Membrane.VKVideo.DeviceServer, nil, name: Membrane.VKVideo.DeviceServer)
  end
end
