defmodule Membrane.VKVideo do
  @moduledoc false
  use Application

  @impl true
  def start(_start_type, _start_args) do
    GenServer.start_link(Membrane.VKVideo.DeviceServer, nil, name: DeviceServer)
  end
end
