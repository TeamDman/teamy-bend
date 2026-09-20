# SPDX-License-Identifier: MPL-2.0
import Base

def view(count: Nat) -> Nat & Image:
  (count, Pix{0})

def tick(events: List<Event>, count: Nat) -> IO(Maybe<Nat>):
  IO.pure(Maybe<Nat>, Some{Nat.add(count, List.length(&1, Event, events))})

def report(result: Maybe<Nat>) -> IO(Unit):
  match result:
    case None{}:
      IO.print("Playback stopped")
    case Some{count}:
      IO.print(String.append("Processed events: ", Nat.show(count)))

def main() -> IO(Unit):
  do IO<Unit>:
    result : Maybe<Nat> <- App.play(~Nat, ~App{view, tick},
      [[Key{65, True{}}], [Move{10, 20}, Mouse{10, 20, 1, False{}}]], 0n)
    report(result)
