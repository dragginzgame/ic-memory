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
5. publishes a capability that can open validated slots.

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

Each record has:

$$
r = (k, s, state, first, last, retiredAt)
$$

where `k` is in `K`, `s` is in `S`, and:

$$
state \in \{\mathsf{Reserved}, \mathsf{Active}, \mathsf{Retired}\}
$$

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
