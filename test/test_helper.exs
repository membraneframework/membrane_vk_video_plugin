opts =
  if System.get_env("CIRCLECI") == "true" do
    [capture_log: true, exclude: [requires_gpu: true]]
  else
    [capture_log: true]
  end

ExUnit.start(opts)
