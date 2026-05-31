/-!
A compact executable model of the allocation-governance invariants described
in the ic-memory whitepaper.

The model is intentionally protocol-level: it proves properties about stable
keys, physical slots, ledger records, validation capabilities, tombstones, and
generation advancement. It does not attempt to verify the Rust implementation.
-/

namespace IcMemory

abbrev StableKey := Nat
abbrev Slot := Fin 255

inductive AllocationState where
  | reserved
  | active
  | retired
  deriving DecidableEq, Repr

structure Record where
  key : StableKey
  slot : Slot
  state : AllocationState
  firstGeneration : Nat
  lastSeenGeneration : Nat
  retiredGeneration : Option Nat
  deriving DecidableEq, Repr

abbrev Ledger := List Record

def ActiveAt (records : Ledger) (key : StableKey) (slot : Slot) : Prop :=
  exists record, List.Mem record records /\
    record.key = key /\ record.slot = slot /\
    record.state = AllocationState.active

def RetiredAt (records : Ledger) (key : StableKey) (slot : Slot) : Prop :=
  exists record, List.Mem record records /\
    record.key = key /\ record.slot = slot /\
    record.state = AllocationState.retired

def NoKeyMovement (records : Ledger) : Prop :=
  forall r1, List.Mem r1 records ->
  forall r2, List.Mem r2 records ->
    r1.key = r2.key -> r1.slot = r2.slot

def NoSlotReuse (records : Ledger) : Prop :=
  forall r1, List.Mem r1 records ->
  forall r2, List.Mem r2 records ->
    r1.slot = r2.slot -> r1.key = r2.key

def NoRetiredRevival (records : Ledger) : Prop :=
  forall key slot, RetiredAt records key slot -> not (ActiveAt records key slot)

structure SafeLedger where
  records : Ledger
  noKeyMovement : NoKeyMovement records
  noSlotReuse : NoSlotReuse records

structure TombstoneLedger extends SafeLedger where
  noRetiredRevival : NoRetiredRevival records

theorem stable_key_has_unique_slot
    (ledger : SafeLedger)
    {r1 r2 : Record}
    (h1 : List.Mem r1 ledger.records)
    (h2 : List.Mem r2 ledger.records)
    (sameKey : r1.key = r2.key) :
    r1.slot = r2.slot :=
  ledger.noKeyMovement r1 h1 r2 h2 sameKey

theorem slot_has_unique_stable_key
    (ledger : SafeLedger)
    {r1 r2 : Record}
    (h1 : List.Mem r1 ledger.records)
    (h2 : List.Mem r2 ledger.records)
    (sameSlot : r1.slot = r2.slot) :
    r1.key = r2.key :=
  ledger.noSlotReuse r1 h1 r2 h2 sameSlot

theorem retired_allocation_cannot_be_active
    (ledger : TombstoneLedger)
    {key : StableKey}
    {slot : Slot}
    (retired : RetiredAt ledger.records key slot) :
    not (ActiveAt ledger.records key slot) :=
  ledger.noRetiredRevival key slot retired

structure Declaration where
  key : StableKey
  slot : Slot
  deriving DecidableEq, Repr

structure OpenAuthority (ledger : SafeLedger) where
  declarations : List Declaration
  sound :
    forall declaration, List.Mem declaration declarations ->
      ActiveAt ledger.records declaration.key declaration.slot

def MayOpen
    {ledger : SafeLedger}
    (authority : OpenAuthority ledger)
    (key : StableKey)
    (slot : Slot) : Prop :=
  exists declaration, List.Mem declaration authority.declarations /\
    declaration.key = key /\ declaration.slot = slot

theorem authority_open_is_backed_by_active_ledger_record
    {ledger : SafeLedger}
    (authority : OpenAuthority ledger)
    {key : StableKey}
    {slot : Slot}
    (openable : MayOpen authority key slot) :
    ActiveAt ledger.records key slot := by
  rcases openable with | intro declaration h =>
  rcases h with | intro inAuthority h =>
  rcases h with | intro sameKey sameSlot =>
  have backed := authority.sound declaration inAuthority
  unfold ActiveAt at backed
  rcases backed with | intro record h =>
  rcases h with | intro inLedger h =>
  rcases h with | intro recordKey h =>
  rcases h with | intro recordSlot active =>
  unfold ActiveAt
  exact Exists.intro record
    (And.intro inLedger
      (And.intro (recordKey.trans sameKey)
        (And.intro (recordSlot.trans sameSlot) active)))

structure GenerationLedger where
  currentGeneration : Nat
  safe : SafeLedger

def stageNextGeneration (ledger : GenerationLedger) : GenerationLedger :=
  { currentGeneration := ledger.currentGeneration + 1
    safe := ledger.safe }

theorem stage_next_generation_strictly_increases
    (ledger : GenerationLedger) :
    ledger.currentGeneration < (stageNextGeneration ledger).currentGeneration := by
  unfold stageNextGeneration
  exact Nat.lt_succ_self ledger.currentGeneration

end IcMemory
