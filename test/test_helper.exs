opts =
  if System.get_env("CIRCLECI") == "true" do
    [capture_log: true, include: :doesnt_require_gpu]
  else
    [capture_log: true]
  end

ExUnit.start(opts)
