defmodule Membrane.VKVideo do
  def start(_, _) do
    GenServer.start_link(Membrane.VKVideo.DeviceServer, nil, name: DeviceServer)
  end
end
