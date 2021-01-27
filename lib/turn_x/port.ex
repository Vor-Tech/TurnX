defmodule TurnX.ServerPort do
  use GenServer
  # ===========================================================================
  # Types
  # ===========================================================================
  defmodule Frame do
    @enforce_keys [:command, :ident, :frame]
    defstruct command: 0x80, ident: nil, frame: [[]]
    @type t :: %__MODULE__{command: byte, ident: non_neg_integer, frame: [list]}
  end

  # ===========================================================================
  # Client callbacks
  # ===========================================================================
  @spec start_link(maybe_improper_list) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(default) when is_list(default) do
    GenServer.start_link(__MODULE__, default)
  end

  @spec command(atom | pid | {atom, any} | {:via, atom, any}, any) :: :ok
  def command(pid, what) do
    GenServer.cast(pid, {:command, what})
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
       arg: arg
     }}
  end

  @impl true
  def handle_cast({:command, what}, state) do
    Port.command(state.port, :erlang.term_to_binary(what))
    {:noreply, state}
  end

  @impl true
  def handle_info({port, {:data, binary}}, state) when port == state.port do
    {:ok, term} = :erlang.binary_to_term(binary, [:safe])
    IO.inspect(term)
    {:noreply, state}
  end
end
