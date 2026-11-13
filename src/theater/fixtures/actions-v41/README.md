# Shared action fixture author

`wire_cases.py` authors 113 synthetic action bodies for two independent callers:
[component lists](../components-v41/README.md) and
[NetworkedActionRequest events](../events-v41/README.md). It never reads parser
output. Each caller exports its own trace from unchanged libfilmshell
`18067987ebc3628ad499c819c6aee814692df43b`; neither caller uses the other's trace
as its oracle.

Cases cover all variants 0–25, optional fields and vector gates, all four handle
generations in variant 0, the unsupported branch of 18 and unknown variants
26–31. The component author adds snapshot/delta masks, sub-states, tails,
maximum-length mixed lists and a valid action followed by a refused one. The
event author adds its references, variant field and a following zoom event.
Exact widths and source values are checked by the two ordinary fixture suites,
including truncation at every bit. This evidence covers this pinned action
grammar; it does not establish other versions or unknown variant layouts.
