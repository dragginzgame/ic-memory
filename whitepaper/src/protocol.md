# Protocol

Each binary contributes a declaration snapshot `D`, containing pairs:

$$
(k,s) \in K \times S
$$

Here `k` is a stable key such as `app.users.v1`, and `s` is a physical
`MemoryManager` ID.

The runtime then:

1. recovers the committed allocation ledger,
2. collects current declarations,
3. validates them against history and policy,
4. commits the next generation,
5. publishes a capability that authorizes the owner's committed-slot open path.

The default runtime performs that sequence before publishing its open
authority. The lower-level Rust APIs expose the pieces separately for framework
owners, so manual integrations must preserve the same order.

The ordering is the central safety boundary:

```text
recover -> validate -> commit -> open
```

Opening application stable-memory handles before this boundary defeats the
protocol.

## State Model

Let `K` be the set of stable keys and `S` be the set of usable physical slots.
A ledger is a finite sequence of records:

$$
L = [r_1,\ldots,r_n]
$$

Each allocation record has:

$$
r = (k, s, state, first, last, retiredAt, schemaHistory)
$$

where `k` is in `K`, `s` is in `S`, and:

$$
state \in \{\mathsf{Reserved}, \mathsf{Active}, \mathsf{Retired}\}
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
\land r.state = \mathsf{Retired}
$$
