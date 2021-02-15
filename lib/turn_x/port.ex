defmodule TurnX.ServerPort do
  use GenServer
  # ===========================================================================
  # Types
  # ===========================================================================
  defmodule Frame do
    @enforce_keys [:command, :frame]
    defstruct command: 0x80, ident: nil, frame: [[]]
    @type t :: %__MODULE__{command: byte, ident: nil | integer, frame: [list]}
  end

  # ===========================================================================
  # Client callbacks
  # ===========================================================================
  @doc """
  Starts an encoder/decoder server and its Rust port.
  """
  @spec start_link(maybe_improper_list) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(default) when is_list(default) do
    GenServer.start_link(__MODULE__, default)
  end

  # TODO: Document this more when we finish the actual Rust media server.
  @doc """
  Sends a `Frame` to the Rust port, if it is still alive.

  Replies with a `Frame`.
  """
  @spec send_frame(atom | pid | {atom, any} | {:via, atom, any}, any) :: map
  def send_frame(pid, what) do
    GenServer.call(pid, {:command, what}, 1000)
  end

  # ===========================================================================
  # Server callbacks
  # ===========================================================================
  @impl true
  def init(arg) do
    {:ok,
     %{
       port:
         Port.open(
           {:spawn_executable,
            Path.join([:code.priv_dir(Mix.Project.config()[:app]), "bin", "turn_x_native"])},
           [
             {:packet, 4},
             :nouse_stdio,
             :binary,
             :exit_status
           ]
         ),
       arg: arg,
       idents: %{}
     }}
  end

  @impl true
  def handle_call({:command, what}, from, state) do
    ident = :erlang.unique_integer()

    Port.command(state.port, :erlang.term_to_binary(%{what | ident: ident}))

    {:noreply,
     %{
       state
       | idents:
           Map.put(state.idents, ident, %{
             :sent => :erlang.monotonic_time(:millisecond),
             :from => from
           })
     }}
  end

  @impl true
  def handle_info({port, {:data, binary}}, state) when port == state.port do
    {:ok, term} = :erlang.binary_to_term(binary, [:safe])

    GenServer.reply(state.idents[term.ident].from, %{
      :frame => term,
      :latency => :erlang.monotonic_time(:millisecond) - state.idents[term.ident].sent
    })

    {:noreply, %{state | idents: Map.delete(state.idents, term.ident)}}
  end
end
