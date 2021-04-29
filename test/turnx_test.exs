defmodule TurnxTest do
  use ExUnit.Case
  doctest Turnx

  test "greets the world" do
    assert Turnx.hello() == :world
  end
end
