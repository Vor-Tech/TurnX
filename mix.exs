defmodule Turnx.MixProject do
  use Mix.Project

  def project do
    [
      app: :turnx,
      version: "0.2.0",
      elixir: "~> 1.11",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Turnx.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      # RATIONALE: Membrane is useful for multimedia
      {:membrane_core, "~> 0.6.1"},
      # RATIONALE: ICE needed for initiating WebRTC
      {:membrane_ice_plugin, "~> 0.4"},
      # RATIONALE: DTLS/SRTP better be used in WebRTC
      {:membrane_dtls_plugin, "~> 0.3"},
      # RATIONALE: These might be used later...WebRTC
      {:membrane_element_udp, "~> 0.3"}, # Init UDP connection
      {:membrane_element_tee, "~> 0.3"}  # Mux and tee (we're an SFU)
    
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"}
    ]
  end
end
