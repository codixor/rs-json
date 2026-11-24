# Conformance suite

`scripts/fetch-corpus.sh` downloads JSONTestSuite (nst) `test_parsing/` here:

- `y_*.json` must parse
- `n_*.json` must fail
- `i_*.json` implementation-defined: record the decision in `decisions.toml`

`tests/conformance.rs` (agent to write) runs every file through **all** entry
points — `parse`, `parse_bytes`, `Parser::parse`, `PullParser<SliceLexer>`,
`PullParser<ReadLexer>` with buffer sizes 1, 7, 64 and 8192 bytes — and
asserts they agree. The 1-byte buffer forces every token across a refill
boundary and is the main test for the spill path.
