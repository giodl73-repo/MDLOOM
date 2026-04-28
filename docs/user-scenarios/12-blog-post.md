# How Raft Consensus Works

Raft is a consensus algorithm designed to be easier to understand than Paxos.
It decomposes consensus into three sub-problems: leader election, log replication,
and safety. Each node is always in one of three states: follower, candidate, or leader.

## Node State Machine

```
raft-states

┌──────────────┐    timeout     ┌──────────────┐
│   Follower   │ ─────────────► │  Candidate   │
│              │                │              │
│  waits for   │ ◄───────────── │  requests    │
│  heartbeats  │  loses election│  votes       │
└──────┬───────┘                └──────┬───────┘
       │                               │
       │ receives                      │ wins
       │ heartbeat                     │ election
       │                               ▼
       │                        ┌──────────────┐
       └────────────────────────│    Leader    │
            steps down          │              │
            on higher term      │  replicates  │
                                │  log entries │
                                └──────────────┘
```

## Log Replication

The leader accepts client requests, appends them to its log, and replicates
to followers. An entry is **committed** once a majority of nodes have written it.

```
log-replication

Client Request
     │
     ▼
┌────────────────────────────────────────────────────┐
│  Leader: append to log                             │
│  ┌─────┬─────┬─────┬─────┬─────┐                  │
│  │  1  │  2  │  3  │  4  │  5  │ ← log entries    │
│  └─────┴─────┴─────┴─────┴─────┘                  │
└────────┬───────────────────────────────────────────┘
         │ AppendEntries RPC (parallel)
    ┌────┴───┐
    │        │
    ▼        ▼
┌───────┐ ┌───────┐
│Follow │ │Follow │  ← majority ACK → committed
│  er 1 │ │  er 2 │
└───────┘ └───────┘
```

## Safety Guarantee

Raft guarantees that if any two entries in different logs have the same
index and term, then the logs are identical in all entries up through that index.
This follows from the **Log Matching Property**.
