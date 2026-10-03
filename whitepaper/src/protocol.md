# Protocol

Each binary contributes a sealed input containing fixed declarations, logical
key requests and explicit host range grants. After recovery and consumer
admission, the runtime resolves that input into a declaration snapshot `D`
containing pairs:

$$
(k,s) \in K \times S
$$

Here `k` is a stable key such as `app.users.v1`, and `s` is a physical
`MemoryManager` ID.

The runtime then:

1. recovers the committed allocation ledger with bounded decoding,
2. runs the host's consumer-admission hook against validated allocation metadata,
3. resolves logical requests and admitted historical keys under current grants,
4. validates the resolved declarations against history and policy,
5. stages and durably persists the next generation,
6. publishes a capability that authorizes the owner's committed-slot open path.

Both owned and default `MemoryRuntime` instances perform that sequence before
publishing open authority. The lower-level Rust APIs expose the pieces
separately for framework owners, so manual integrations must preserve the same
order.

The ordering is the central safety boundary:

```text
recover -> prepare -> resolve -> validate -> stage -> persist -> publish -> open
```

Opening application stable-memory handles before this boundary defeats the
protocol.

## Logical Placement and Consumer Admission

A known key retains its committed physical slot. A new key takes the lowest
unclaimed ID in an explicit `Allowed` host grant, in canonical stable-key order.
Fixed claims, governance, reservations, omitted allocations and tombstones
remain occupied. Custom policy may reject placement; it cannot silently move
known keys or create an undeclared eligible pool.

The host's `RuntimeBootstrapPolicy::prepare_bootstrap` hook can inspect bounded
recovered allocation metadata, reject a consumer identity transition, and select
known historical keys before resolution. Selection requires current host grants
and rejects unknown or retired keys. Metadata supplies no store handles or
application payloads. Consumer journal-debt, accepted-schema and retirement
checks remain outside allocation governance.

Cold runtime reconstruction repeats admission and advances one generation after
successful persistence. Matching warm bootstrap returns the existing capability
without another commit. A library joining a warm host verifies its requirements
with `verify_authority` and opens committed keys; it does not replay admission
or replace the host's policy or bucket geometry. Omission retains ownership but
removes a key from current open authority unless explicitly admitted again.

## State Model

Let `K` be the set of stable keys and `S` be the set of usable physical slots.
A ledger is a finite sequence of records:

$$
L = [r_1,\ldots,r_n]
$$

Each allocation record has:

$$
r = (k, s, state, first, last, schemaHistory)
$$

where `k` is in `K`, `s` is in `S`, and:

$$
state \in \{\mathsf{Reserved}, \mathsf{Active}, \mathsf{Retired}(g_r)\}
$$

`schemaHistory` is diagnostic metadata observed across committed generations.
Today it records an optional nonzero in-place schema version. It helps humans
and framework tooling understand which schema was declared when, but it is not
used to prove application data compatibility.

Committed ledgers also carry generation records. Each generation record stores
the committed generation number, its mandatory parent generation, an optional
runtime fingerprint, a declaration count, and an optional integration-supplied
commit timestamp. The first real staged generation has parent `0`; an empty
genesis ledger is generation `0` and has no generation record.

## Active Binding

A stable key `k` is active at slot `s` in ledger `L`, written
`ActiveAt(L,k,s)`, when:

$$
\exists r \in L.\; r.key = k \land r.slot = s
\land r.state = \mathsf{Active}
$$

## Retired Binding

A stable key `k` is retired at slot `s`, written `RetiredAt(L,k,s)`, when:

$$
\exists r \in L.\; r.key = k \land r.slot = s
\land r.state = \mathsf{Retired}(g_r)
$$

The retirement generation `g_r` is part of the retired lifecycle state rather
than separate nullable metadata. Therefore a retired record without a
retirement generation, and a live record with one, cannot be represented.
